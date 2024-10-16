use crate::{METADATA, PAUSABLE_DATA};
use alloc::vec;
use alloc::vec::Vec;
use ckb_ssri_sdk::public_module_traits::{
    UDTExtended, UDTMetadata, UDTPausable, UDTPausableData, UDT,
};
use ckb_ssri_sdk::{ssri_method, ssri_module};
use ckb_std::ckb_types::packed::{Byte32, Script};
use serde_molecule::{from_slice, to_vec};
#[ssri_module]
pub struct UDTSSRI;

impl UDT for UDTSSRI {
    #[ssri_method(level = "cell")]
    fn balance() -> Result<u128, SSRIError> {
        Err(SSRIError::SSRIMethodsNotImplemented)
    }
    #[ssri_method(level = "transaction", transaction = true)]
    fn transfer(from: Script, to: Script, amount: u128) -> Result<(), SSRIError> {
        Err(SSRIError::SSRIMethodsNotImplemented)
    }
}

#[ssri_module(base=UDT)]
impl UDTMetadata for UDTSSRI {
    /** Note: If the UDT is issued with a generic UDT Type and defines it's metadata in CellDep, it would require Chain level; if it is only compliant to the SSRI trait UDT and is able to return name/symbol/decimals within the script, and it would require only code/script level. */
    #[ssri_method(level = "code")]
    fn name() -> Result<Bytes, SSRIError> {
        return Ok(Bytes::from(METADATA.name.as_bytes()));
    }
    #[ssri_method(level = "code")]
    fn symbol() -> Result<Bytes, SSRIError> {
        return Ok(Bytes::from(METADATA.symbol.as_bytes()));
    }
    #[ssri_method(level = "code")]
    /* Note: By default, decimals are 8 when decimals() are not implemented */
    fn decimals() -> Result<u8, SSRIError> {
        return Ok(METADATA.decimals);
    }
}

#[ssri_module(base=UDT)]
impl UDTExtended for UDTSSRI {
    #[ssri_method(level = "transaction", transaction = true)]
    fn mint(lock: Script, amount: u128) -> Result<(), SSRIError> {
        todo!()
    }

    #[ssri_method(implemented = false)]
    fn approve(
        tx: Optional<RawTransaction>,
        spender: Script,
        amount: u128,
    ) -> Result<(), SSRIError> {
        Err(SSRIError::SSRIMethodsNotImplemented)
    }

    #[ssri_method(implemented = false)]
    fn allowance(owner: Script, spender: Script) -> Result<u128, SSRIError> {
        Err(SSRIError::SSRIMethodsNotImplemented)
    }

    #[ssri_method(implemented = false)]
    fn increase_allowance(
        tx: Optional<RawTransaction>,
        spender: Script,
        added_value: u128,
    ) -> Result<(), SSRIError> {
        Err(SSRIError::SSRIMethodsNotImplemented)
    }

    #[ssri_method(implemented = false)]
    fn decrease_allowance(
        tx: Optional<RawTransaction>,
        spender: Script,
        subtracted_value: u128,
    ) -> Result<(), SSRIError> {
        Err(SSRIError::SSRIMethodsNotImplemented)
    }
}

#[ssri_module(base=UDT)]
impl UDTPausable for UDTSSRI {
    #[ssri_method(level = "transaction", transaction = true)]
    fn pause(
        tx: Optional<RawTransaction>,
        lock_hashes: &Vec<[u8; 32]>,
    ) -> Result<RawTransaction, SSRIError> {
        todo!()
    }

    #[ssri_method(level = "transaction", transaction = true)]
    fn unpause(
        tx: Optional<RawTransaction>,
        lock_hashes: &Vec<[u8; 32]>,
    ) -> Result<RawTransaction, SSRIError> {
        todo!()
    }

    #[ssri_method(level = "transaction", transaction = true)]
    fn is_paused(lock_hashes: &Vec<[u8; 32]>) -> Result<bool, SSRIError> {
        let mut current_pause_list: Optional<&[u8; 32]> = Some(&PAUSABLE_DATA.pause_list);
        while true {
            match current_pause_list {
                Some(pause_list) => {
                    for lock_hash in lock_hashes {
                        if pause_list.contains(lock_hash) {
                            return Ok(true);
                        }
                    }
                    match PAUSABLE_DATA.next_type_hash {
                        Some(next_type_hash) => {
                            let mut index = 0;
                            while let Ok(type_hash) = load_cell_tpe_hash(index, Source::CellDep) {
                                if type_hash == next_type_hash {
                                    let next_data =
                                        from_slice(load_cell_data(index, Source::CellDep), false);
                                    current_pause_list = Some(&next_data.pause_list);
                                }
                            }
                        }
                        None => {
                            return Ok(false);
                        }
                    }
                }
                None => {
                    return Ok(false);
                }
            }
        }
    }

    #[ssri_method(level = "transaction", transaction = true)]
    fn enumerate_paused() -> Result<&Vec<[u8; 32]>, SSRIError> {
        let mut aggregated_paused_list: Vec<[u8; 32]> = vec![];
        aggregated_paused_list.extend(&PAUSABLE_DATA.pause_list.clone());
        while true {
            match PAUSABLE_DATA.next_type_hash {
                Some(next_type_hash) => {
                    let mut index = 0;
                    while let Ok(type_hash) = load_cell_tpe_hash(index, Source::CellDep) {
                        if type_hash == next_type_hash {
                            let next_data: UDTPausableData =
                                from_slice(load_cell_data(index, Source::CellDep), false)?;
                                aggregated_paused_list.extend(&next_data.pause_list);
                        }
                    }
                }
                None => {
                    return Ok(&aggregated_paused_list);
                }
            }
        }
    }
}
