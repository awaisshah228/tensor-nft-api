use std::sync::{Arc, Mutex};

use actix_web::{
    delete, get, post, put,
    web::{Data, Json, Path, Query, ServiceConfig},
    HttpResponse, Responder,
};
use serde::{Deserialize, Serialize};
// use solana_client::rpc_client::RpcClient;
use crate::rpc;
use solana_sdk::pubkey::Pubkey;
use std::fs::File;
use std::str::FromStr;
use utoipa::{IntoParams, ToSchema};
//use crate::rpc::RpcClient;
use crate::rpc::rpc_client::RpcClient;
use solana_sdk::borsh0_10::try_from_slice_unchecked;

use spl_token_metadata::state::Metadata;

use crate::{LogApiKey, RequireApiKey};
use anyhow::Result;

const METAPLEX_PROGRAM_ID: &'static str = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s";

#[derive(Default)]
pub(super) struct TodoStore {
    todos: Mutex<Vec<Todo>>,
}

pub(super) fn configure(store: Data<TodoStore>) -> impl FnOnce(&mut ServiceConfig) {
    |config: &mut ServiceConfig| {
        config
            .app_data(store)
            // .service(search_todos)
            // .service(get_todos)
            // .service(create_todo)
            // .service(delete_todo)
            // .service(get_todo_by_id)
            // .service(update_todo)
            .service(get_nft_by_id)
            .service(get_nft_metadata);
    }
}



/// Non-Fungible Token (NFT).
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct NFT {
    /// Unique identifier for the NFT.
    #[schema(example = 1)]
    id: u64,
    /// Name of the NFT.
    #[schema(example = "My NFT")]
    name: String,
    /// Description of the NFT.
    #[schema(example = "A unique digital asset representing ownership of a digital artwork.")]
    description: String,
    // Add any other fields specific to your NFT definition
}

#[derive(Debug, Serialize)]
pub struct JSONCreator {
    pub address: String,
    pub verified: bool,
    pub share: u8,
}

#[derive(Debug, Serialize)]
pub struct NFTMetadata {
    pub name: String,
    pub symbol: String,
    pub seller_fee_basis_points: u16,
    pub uri: String,
    pub creators: Vec<JSONCreator>,
}


/// Todo endpoint error responses
#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub(super) enum ErrorResponse {
    /// When Todo is not found by search term.
    NotFound(String),
    /// When there is a conflict storing a new todo.
    Conflict(String),
    /// When todo endpoint was called without correct credentials
    Unauthorized(String),
}

/// Get nft  by given nft id.
///
/// Return found `Todo` with status 200 or 404 not found if `Todo` is not found from shared in-memory storage.
#[utoipa::path(
    responses(
        (status = 200, description = "NFT found by ID", body = NFT),
        (status = 404, description = "NFT not found by ID", body = ErrorResponse, example = json!(ErrorResponse::NotFound(String::from("id = 1"))))
    ),
    params(
        ("id", description = "Unique ID of the NFT")
    )
)]
#[get("/nft/{id}")]
pub(super) async fn get_nft_by_id(id: Path<u64>) -> impl Responder {
    // let nfts = nft_store.nfts.lock().unwrap();
    let nfts = vec![
        NFT {
            id: 1,
            name: String::from("Dummy NFT 1"),
            description: String::from("Description of Dummy NFT 1"),
        },
        NFT {
            id: 2,
            name: String::from("Dummy NFT 2"),
            description: String::from("Description of Dummy NFT 2"),
        },
        // Add more dummy NFTs if needed
    ];
    let id = id.into_inner();

    nfts.iter()
        .find(|nft| nft.id == id)
        .map(|nft| HttpResponse::Ok().json(nft))
        .unwrap_or_else(|| {
            HttpResponse::NotFound().json(ErrorResponse::NotFound(format!("id = {}", id)))
        })
}

#[utoipa::path(
    responses(
        (status = 200, description = "NFT found by mint", body = NFT),
        (status = 404, description = "NFT not found by mint", body = ErrorResponse, example = json!(ErrorResponse::NotFound(String::from("mint = <mint_address>"))))
    ),
    params(
        ("mint_account", description = "Mint address of the NFT")
    )
)]
#[get("/nft/metadata/{mint_account}")]
pub async fn get_nft_metadata(mint_account: Path<String>) -> impl Responder {
    match fetch_metadata(&mint_account).await {
        Ok(metadata) => HttpResponse::Ok().json(metadata),
        Err(err) => {
            eprintln!("Error fetching metadata: {:?}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn fetch_metadata(mint_account: &str) -> Result<NFTMetadata> {
    // let connection = RpcClient::new("https://flashtr-flash-885f.mainnet.rpcpool.com/11a75a74-fd8e-44cc-87f4-d84bb82d0983".to_string());
    let rpc_client = Arc::new(rpc::create_rpc_client(
        "https://api.mainnet-beta.solana.com".to_string(),
    ));

    let mint_pubkey = Pubkey::from_str(&mint_account)?;

    let metadata_pda = match get_metadata_pda(mint_pubkey) {
        Some(pubkey) => pubkey,
        None => return Err(anyhow::anyhow!("No metaplex account found")),
    };
    // let current_epoch = rpc::get_current_epoch(rpc_client.clone()).await;
    // println!("{}",current_epoch);

    let account_data = rpc_client.get_account_data(&metadata_pda).await?;

    println!("Accoutn data {:?}", account_data);

    let metadata: Metadata = my_try_from_slice_unchecked(&account_data)?;

    let creators: Vec<JSONCreator> = metadata
        .data
        .creators
        .unwrap_or_default()
        .iter()
        .map(|c| JSONCreator {
            address: c.address.to_string(),
            verified: c.verified,
            share: c.share,
        })
        .collect();

    Ok(NFTMetadata {
        name: metadata
            .data
            .name
            .to_string()
            .trim_matches(char::from(0))
            .to_owned(),
        symbol: metadata
            .data
            .symbol
            .to_string()
            .trim_matches(char::from(0))
            .to_owned(),
        seller_fee_basis_points: metadata.data.seller_fee_basis_points,
        uri: metadata
            .data
            .uri
            .to_string()
            .trim_matches(char::from(0))
            .to_owned(),
        creators,
    })
}

fn get_metadata_pda(mint_account: Pubkey) -> Option<Pubkey> {
    let metaplex_pubkey = METAPLEX_PROGRAM_ID
        .parse::<Pubkey>()
        .expect("Failed to parse Metaplex Program Id");

    let seeds = &[
        "metadata".as_bytes(),
        metaplex_pubkey.as_ref(),
        mint_account.as_ref(),
    ];

    let (pda, _) = Pubkey::find_program_address(seeds, &metaplex_pubkey);
    Some(pda)
}
