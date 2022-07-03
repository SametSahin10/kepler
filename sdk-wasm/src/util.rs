use kepler_lib::{
    ssi::cacao_zcap::{
        cacaos::{
            siwe::Message,
            siwe_cacao::{SIWESignature, SignInWithEthereum},
            BasicSignature, CACAO,
        },
        translation::cacao_to_zcap::{cacao_to_zcap, CacaoToZcapError},
    },
    zcap::KeplerDelegation,
};

pub fn make_orbit_id_pkh_eip155(address: String, chain_id: u32, name: Option<String>) -> String {
    make_orbit_id(format!("pkh:eip155:{}:{}", chain_id, address), name)
}

fn make_orbit_id(did_suffix: String, name: Option<String>) -> String {
    format!(
        "kepler:{}://{}",
        did_suffix,
        name.unwrap_or_else(|| String::from("default"))
    )
}

pub fn siwe_to_zcap(
    message: Message,
    signature: BasicSignature<SIWESignature>,
) -> Result<KeplerDelegation, CacaoToZcapError> {
    cacao_to_zcap(&CACAO::<SignInWithEthereum>::new(message.into(), signature))
}