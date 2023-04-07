//! Configuration

use std::time::Duration;

use eyre::eyre;

use hyperlane_base::{
    decl_settings, CheckpointSyncerConf, RawCheckpointSyncerConf, RawSignerConf, Settings,
    SignerConf,
};
use hyperlane_core::config::*;
use hyperlane_core::HyperlaneDomain;

decl_settings!(Validator,
    Parsed {
        // The name of the origin chain
        origin_chain: HyperlaneDomain,
        /// The validator attestation signer
        validator: SignerConf,
        /// The checkpoint syncer configuration
        checkpoint_syncer: CheckpointSyncerConf,
        /// The reorg_period in blocks
        reorg_period: u64,
        /// How frequently to check for new checkpoints
        interval: Duration,
    },
    Raw {
        originchainname: Option<String>,
        validator: Option<RawSignerConf>,
        checkpointsyncer: Option<RawCheckpointSyncerConf>,
        reorgperiod: Option<StrOrInt>,
        interval: Option<StrOrInt>,
    },
);

impl FromRawConf<'_, RawValidatorSettings> for ValidatorSettings {
    fn from_config_filtered(
        raw: RawValidatorSettings,
        cwp: &ConfigPath,
        _filter: (),
    ) -> ConfigResult<Self> {
        let mut err = ConfigParsingError::default();

        let validator = raw
            .validator
            .ok_or_else(|| eyre!("Missing `validator`"))
            .take_err(&mut err, || cwp + "validator")
            .and_then(|r| {
                r.parse_config(&cwp.join("validator"))
                    .take_config_err(&mut err)
            });

        let checkpoint_syncer = raw
            .checkpointsyncer
            .ok_or_else(|| eyre!("Missing `checkpointsyncer`"))
            .take_err(&mut err, || cwp + "checkpointsyncer")
            .and_then(|r| {
                r.parse_config(&cwp.join("checkpointsyncer"))
                    .take_config_err(&mut err)
            });

        let reorg_period = raw
            .reorgperiod
            .ok_or_else(|| eyre!("Missing `reorgperiod`"))
            .take_err(&mut err, || cwp + "reorgperiod")
            .and_then(|r| r.try_into().take_err(&mut err, || cwp + "reorgperiod"));

        let interval = raw
            .interval
            .ok_or_else(|| eyre!("Missing `interval`"))
            .take_err(&mut err, || cwp + "interval")
            .and_then(|r| {
                r.try_into()
                    .map(Duration::from_secs)
                    .take_err(&mut err, || cwp + "interval")
            });

        let Some(origin_chain_name) = raw
            .originchainname
            .ok_or_else(|| eyre!("Missing `originchainname`"))
            .take_err(&mut err, || cwp + "originchainname")
        else { return Err(err) };

        let base = raw
            .base
            .parse_config_with_filter::<Settings>(
                cwp,
                Some(&[&*origin_chain_name].into_iter().collect()),
            )
            .take_config_err(&mut err);

        let origin_chain = if let Some(base) = &base {
            base.lookup_domain(&origin_chain_name)
                .take_err(&mut err, || cwp + "originchainname")
        } else {
            None
        };

        if err.is_empty() {
            Ok(Self {
                base: base.unwrap(),
                origin_chain: origin_chain.unwrap(),
                validator: validator.unwrap(),
                checkpoint_syncer: checkpoint_syncer.unwrap(),
                reorg_period: reorg_period.unwrap(),
                interval: interval.unwrap(),
            })
        } else {
            Err(err)
        }
    }
}
