//! Program state
use {
    borsh::{BorshDeserialize, BorshSchema, BorshSerialize},
    solana_program::{program_pack::IsInitialized, pubkey::Pubkey},
};

/// Struct provided metadata of the related program
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct RelyingPartyData {
    /// Struct version, allows for upgrades to the program
    pub version: u8,
    /// The account allowed to update the data
    pub authority: Pubkey,
    /// The metadata of the corresponded program to be added
    pub related_program: Pubkey,
    /// The metadata of the related program
    pub related_program_data: RelatedProgramInfo,
}

/// Metadata of the some program to show for Vaccount
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct RelatedProgramInfo {
    /// Name of the program to show in Vaccount
    pub name: String,
    /// Icon content identifier
    pub icon_cid: [u8; Self::ICON_CID_SIZE],
    /// Domain name of the related program
    pub domain_name: String,
    /// Allowed redirect URI for Vaccount in program
    pub redirect_uri: Vec<String>
}

impl RelatedProgramInfo {
    /// https://pascalprecht.github.io/posts/content-identifiers-in-ipfs
    pub const ICON_CID_SIZE: usize = 64;
}

impl RelyingPartyData {
    /// Version to fill in on new created accounts
    pub const CURRENT_VERSION: u8 = 1;
}

impl IsInitialized for RelyingPartyData {
    /// Is initialized
    fn is_initialized(&self) -> bool {
        self.version == Self::CURRENT_VERSION
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::convert::TryInto;
    /// Version for tests
    pub const TEST_VERSION: u8 = 1;
    /// Pubkey for tests
    pub const TEST_AUTHORITY_PUBKEY: Pubkey = Pubkey::new_from_array([100; 32]);
    /// Pubkey for tests
    pub const TEST_RELATED_PROGRAM_PUBKEY: Pubkey = Pubkey::new_from_array([100; 32]);
    /// Related program name
    #[test]
    fn serialize_desialize_data() {
        let related_program_data: RelatedProgramInfo = RelatedProgramInfo {
            name: String::from("test_program"),
            icon_cid: "d2a84f4b8b650937ec8f73cd8be2c74add5a911ba64df27458ed8229da804a26".as_bytes().try_into().unwrap(),
            domain_name: String::from("http://localhost:8989/"),
            redirect_uri: vec![
                "https://velas.com/ru".to_string(),
                "https://wallet.velas.com/".to_string(),
            ],
        };

        let relying_party_data: RelyingPartyData = RelyingPartyData {
            version: TEST_VERSION,
            authority: TEST_AUTHORITY_PUBKEY,
            related_program: TEST_AUTHORITY_PUBKEY,
            related_program_data: related_program_data,
        };

        let packed = relying_party_data.try_to_vec().unwrap();
        let unpacked = RelyingPartyData::try_from_slice(packed.as_slice()).unwrap();

        assert_eq!(relying_party_data, unpacked);
    }
}
