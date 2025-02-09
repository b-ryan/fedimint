use std::{collections::BTreeMap, io};

use fedimint_api::core::{
    ConsensusItem, Input, ModuleKey, Output, OutputOutcome, PendingOutput, PluginConsensusItem,
    PluginDecode, PluginInput, PluginOutput, PluginOutputOutcome, PluginPendingOutput,
    PluginSpendableOutput, SpendableOutput,
};
use fedimint_api::{
    encoding::{Decodable, DecodeError, Encodable},
    Amount,
};

pub const MINT_MODULE_KEY: u16 = 0;

// TODO: DELME
#[derive(Default, Clone)]
pub struct MintModuleDecoder;

impl PluginDecode for MintModuleDecoder {
    fn decode_spendable_output(mut d: &mut dyn io::Read) -> Result<SpendableOutput, DecodeError> {
        Ok(SpendableOutput::from(
            MintSpendableOutput::consensus_decode(&mut d, &BTreeMap::<_, ()>::new())?,
        ))
    }

    fn decode_pending_output(mut d: &mut dyn io::Read) -> Result<PendingOutput, DecodeError> {
        Ok(PendingOutput::from(MintPendingOutput::consensus_decode(
            &mut d,
            &BTreeMap::<_, ()>::new(),
        )?))
    }

    fn decode_output(mut d: &mut dyn io::Read) -> Result<Output, DecodeError> {
        Ok(Output::from(MintOutput::consensus_decode(
            &mut d,
            &BTreeMap::<_, ()>::new(),
        )?))
    }
    fn decode_output_outcome(mut d: &mut dyn io::Read) -> Result<OutputOutcome, DecodeError> {
        Ok(OutputOutcome::from(MintOutputOutcome::consensus_decode(
            &mut d,
            &BTreeMap::<_, ()>::new(),
        )?))
    }

    fn decode_input(mut d: &mut dyn io::Read) -> Result<Input, DecodeError> {
        Ok(Input::from(MintInput::consensus_decode(
            &mut d,
            &BTreeMap::<_, ()>::new(),
        )?))
    }

    fn decode_consensus_item(
        mut r: &mut dyn io::Read,
    ) -> Result<fedimint_api::core::ConsensusItem, DecodeError> {
        Ok(ConsensusItem::from(MintConsensusItem::consensus_decode(
            &mut r,
            &BTreeMap::<_, ()>::new(),
        )?))
    }
}

#[derive(Encodable, Decodable, Clone)]
pub struct MintOutput;

impl PluginOutput for MintOutput {
    fn module_key(&self) -> ModuleKey {
        MINT_MODULE_KEY
    }

    fn amount(&self) -> Amount {
        todo!()
    }
}

#[derive(Encodable, Decodable, Clone)]
pub struct MintPendingOutput;

impl PluginPendingOutput for MintPendingOutput {
    fn module_key(&self) -> ModuleKey {
        MINT_MODULE_KEY
    }

    fn amount(&self) -> Amount {
        todo!()
    }
}

#[derive(Encodable, Decodable, Clone)]
pub struct MintOutputOutcome;

impl PluginOutputOutcome for MintOutputOutcome {
    fn module_key(&self) -> ModuleKey {
        MINT_MODULE_KEY
    }
}

#[derive(Encodable, Decodable, Clone)]
pub struct MintSpendableOutput;

impl PluginSpendableOutput for MintSpendableOutput {
    fn module_key(&self) -> ModuleKey {
        todo!()
    }

    fn amount(&self) -> Amount {
        todo!()
    }

    fn key(&self) -> String {
        todo!()
    }
}

#[derive(Encodable, Decodable, Clone)]
pub struct MintInput;

impl PluginInput for MintInput {
    fn module_key(&self) -> ModuleKey {
        MINT_MODULE_KEY
    }

    fn amount(&self) -> Amount {
        todo!()
    }
}

#[derive(Encodable, Decodable, Clone)]
pub struct MintConsensusItem;

impl PluginConsensusItem for MintConsensusItem {
    fn module_key(&self) -> ModuleKey {
        MINT_MODULE_KEY
    }
}
