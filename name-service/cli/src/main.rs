use {
    clap::{
        crate_description, crate_name, crate_version, value_t, value_t_or_exit, App, AppSettings, Arg,
        SubCommand,
    },
    solana_clap_utils::{
        input_parsers::pubkey_of,
        input_validators::{is_keypair, is_url, is_valid_pubkey, is_amount},
    },
    solana_client::rpc_client::RpcClient,
    solana_sdk::{
        commitment_config::CommitmentConfig,
        pubkey::Pubkey,
        signature::{read_keypair_file, Keypair, Signer},
        transaction::Transaction,
        hash::hashv,
        system_instruction,
        program_pack::Pack,
        native_token::lamports_to_sol,
        bs58,
    },
    spl_name_service::{
        state::{NameRecordHeader, get_seeds_and_key},
        instruction::{NameRegistryInstruction},
        id,
    },
    // vpl_name_register::{
    //     instruction::{issue_subname, issue_top_level_name, set_top_level_name_issuer, NameIssuerInstruction},
    //     get_name_issuer_class,
    //     get_tln_issuer_store,
        
    // },
    borsh::{BorshDeserialize, BorshSerialize},
    std::{cmp::min, str::FromStr},
    cid::Cid,
    std::convert::TryFrom,
};

struct Config {
    fee_payer: Keypair,
    authority: Keypair,
    json_rpc_url: String,
    verbose: bool,
    dry_run: bool,
}

fn send_transaction(
    rpc_client: &RpcClient,
    config: &Config,
    transaction: Transaction,
) -> solana_client::client_error::Result<()> {
    if config.dry_run {
        let result = rpc_client.simulate_transaction(&transaction)?;
        println!("Simulate result: {:?}", result);
    } else {
        let signature = rpc_client.send_and_confirm_transaction_with_spinner(&transaction)?;
        println!("Signature: {}", signature);
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_matches = App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg({
            let arg = Arg::with_name("config_file")
                .short("C")
                .long("config")
                .value_name("PATH")
                .takes_value(true)
                .global(true)
                .help("Configuration file to use");
            if let Some(ref config_file) = *solana_cli_config::CONFIG_FILE {
                arg.default_value(&config_file)
            } else {
                arg
            }
        })
        .arg(
            Arg::with_name("fee_payer")
                .long("fee-payer")
                .value_name("KEYPAIR")
                .validator(is_keypair)
                .takes_value(true)
                .global(true)
                .help("Filepath or URL to a fee-payer keypair [default: client keypair]"),
        )
        .arg(
            Arg::with_name("verbose")
                .long("verbose")
                .short("v")
                .takes_value(false)
                .global(true)
                .help("Show additional information"),
        )
        .arg(
            Arg::with_name("json_rpc_url")
                .long("url")
                .value_name("URL")
                .takes_value(true)
                .global(true)
                .validator(is_url)
                .help("JSON RPC URL for the cluster [default: value from configuration file]"),
        )
        .arg(
            Arg::with_name("authority")
                .long("authority")
                .value_name("KEYPAIR")
                .validator(is_keypair)
                .takes_value(true)
                .global(true)
                .help(
                    "Filepath or URL to a name record authority keypair. \
                    Name record authority needs to change name record account state. \
                    [default: client keypair]",
                ),
        )
        .subcommand(
            SubCommand::with_name("account")
                .about("Display information stored in name record")
                .arg(
                    Arg::with_name("name_record")
                        .value_name("NAME_RECORD_ADDRESS")
                        .validator(is_valid_pubkey)
                        .index(1)
                        .required(true)
                        .help("The address of the name record."),
                ),
        )
        .subcommand(
            SubCommand::with_name("create-record")
                .about("Create an empty name record")
                .arg(
                    Arg::with_name("name")
                        .value_name("NAME")
                        .validator(is_valid_account_name)
                        .index(1)
                        .required(true)
                        .help("Account name blablabla"),
                )
                .arg(
                    Arg::with_name("owner")
                        .long("owner")
                        .value_name("OWNER_ADDRESS")
                        .validator(is_valid_pubkey)
                        .help(
                            "The owner of the name record that can modify state"
                        ),
                )
                .arg(
                    Arg::with_name("lamports")
                        .long("lamports")
                        .value_name("AMOUNT")
                        .validator(is_amount)
                        .help("Lamports to create a new name record. [default: rent_exempt_value]"),
                )
                .arg(
                    Arg::with_name("space")
                        .long("space")
                        .value_name("AMOUNT")
                        .validator(is_amount)
                        .help("Number of bytes of memory to allocate in addition to the base name record information"),
                )
                .arg(
                    Arg::with_name("class_name")
                        .long("class_name")
                        .value_name("KEYPAIR")
                        .validator(is_keypair)
                        .help("The class of data this account represents (DNS record, twitter handle, SPL Token name/symbol, etc)"),
                )
                .arg(
                    Arg::with_name("parent_name")
                        .long("parent-name")
                        .value_name("PARENT_NAME_ADDRESS")
                        .validator(is_valid_pubkey)
                        .help("Name contains the account address of the parent name"),
                )
                .arg(
                    Arg::with_name("parent_owner")
                        .long("parent-owner")
                        .value_name("KEYPAIR")
                        .validator(is_keypair)
                        .requires("parent_name")
                        .help("Owner of the parent name record"),
                )
        )
        .subcommand(
            SubCommand::with_name("wcreate-record")
                .about("Create an empty name record")
                .arg(
                    Arg::with_name("name")
                        .value_name("NAME")
                        .validator(is_valid_account_name)
                        .index(1)
                        .required(true)
                        .help("Account name blablabla"),
                )
                .arg(
                    Arg::with_name("owner")
                        .long("owner")
                        .value_name("OWNER_ADDRESS")
                        .validator(is_valid_pubkey)
                        .help(
                            "The owner of the name record that can modify state"
                        ),
                )
                .arg(
                    Arg::with_name("lamports")
                        .long("lamports")
                        .value_name("AMOUNT")
                        .validator(is_amount)
                        .help("Lamports to create a new name record. [default: rent_exempt_value]"),
                )
                .arg(
                    Arg::with_name("space")
                        .long("space")
                        .value_name("AMOUNT")
                        .validator(is_amount)
                        .help("Number of bytes of memory to allocate in addition to the base name record information"),
                )
                .arg(
                    Arg::with_name("class_name")
                        .long("class_name")
                        .value_name("KEYPAIR")
                        .validator(is_keypair)
                        .help("The class of data this account represents (DNS record, twitter handle, SPL Token name/symbol, etc)"),
                )
                .arg(
                    Arg::with_name("parent_name")
                        .long("parent-name")
                        .value_name("PARENT_NAME_ADDRESS")
                        .validator(is_valid_pubkey)
                        .help("Name contains the account address of the parent name"),
                )
                .arg(
                    Arg::with_name("parent_owner")
                        .long("parent-owner")
                        .value_name("KEYPAIR")
                        .validator(is_keypair)
                        .requires("parent_name")
                        .help("Owner of the parent name record"),
                )
        )
        .subcommand(
            SubCommand::with_name("update-record")
                .about("Update the data in a name record.")
                .arg(
                    Arg::with_name("name_record")
                        .value_name("NAME_RECORD_ADDRESS")
                        .validator(is_valid_pubkey)
                        .index(1)
                        .required(true)
                        .help(
                            "The address of the name record to update"
                        ),
                )
                .arg(
                    Arg::with_name("data")
                        .value_name("DATA")
                        .validator(is_base58)
                        .index(2)
                        .required(true)
                        .help("The data to be write in name record in base58 format"),
                )
                .arg(
                    Arg::with_name("offset")
                        .value_name("OFFSET")
                        .validator(is_amount)
                        .help("The offset for the data."),
                )
        )
        .subcommand(
            SubCommand::with_name("transfer-ownership")
                .about("Transfer ownership of a name record")
                .arg(
                    Arg::with_name("name_record")
                        .value_name("NAME_RECORD_ADDRESS")
                        .validator(is_valid_pubkey)
                        .index(1)
                        .required(true)
                        .help(
                            "The address of the name record to update"
                        ),
                )
                .arg(
                    Arg::with_name("new_owner")
                        .value_name("NEW_OWNER_ADDRESS")
                        .validator(is_valid_pubkey)
                        .index(2)
                        .required(true)
                        .help("The new owner of the name record"),
                )
                .arg(
                    Arg::with_name("class_account")
                        .long("class-account")
                        .value_name("KEYPAIR")
                        .validator(is_keypair)
                        .help("Filepath or URL to a class-account keypair"),
                )
            )
        .subcommand(
            SubCommand::with_name("delete-record")
                .about("Transfer ownership of a name record")
                .arg(
                    Arg::with_name("name_record")
                        .value_name("NAME_RECORD_ACCOUNT")
                        .validator(is_valid_pubkey)
                        .index(1)
                        .required(true)
                        .help("The address of the n name record owner"),
                )
                .arg(
                    Arg::with_name("refund_account")
                        .value_name("REFUND_ACCOUNT")
                        .validator(is_valid_pubkey)
                        .index(2)
                        .required(true)
                        .help("Lamports remaining in the name record will be transferred to the refund account"),
                )
        )
        .subcommand(
            SubCommand::with_name("register-parent-name")
                .about("Registrate a new parent name with price for child.")
                .arg(
                    Arg::with_name("name")
                        .value_name("NAME")
                        .validator(is_valid_account_name)
                        .index(1)
                        .required(true)
                        .help("Account name blablabla"),
                )
                .arg(
                    Arg::with_name("price")
                        .value_name("PRICE")
                        .validator(is_amount)
                        .index(2)
                        .required(true)
                        .help("Price for child of parent name"),
                )
                .arg(
                    Arg::with_name("owner")
                        .long("owner")
                        .value_name("OWNER_ADDRESS")
                        .validator(is_valid_pubkey)
                        .help(
                            "The owner of the name record that can modify state"
                        ),
                )
        )
        .subcommand(
            SubCommand::with_name("set-tln-issuer")
                .about("Set top level name issuer in NameIssuer.")
                .arg(
                    Arg::with_name("tln_issuer")
                        .long("tln_issuer")
                        .value_name("TLN_ISSUER")
                        .validator(is_valid_pubkey)
                        .help(
                            "The owner of the name record that can modify state"
                        ),
                )
        )
        .get_matches();

    let (sub_command, sub_matches) = app_matches.subcommand();
    let matches = sub_matches.unwrap();

    let config = {
        let cli_config = if let Some(config_file) = matches.value_of("config_file") {
            solana_cli_config::Config::load(config_file).unwrap_or_default()
        } else {
            solana_cli_config::Config::default()
        };

        Config {
            json_rpc_url: matches
                .value_of("json_rpc_url")
                .unwrap_or(&cli_config.json_rpc_url)
                .to_string(),
            fee_payer: read_keypair_file(
                matches
                    .value_of("fee_payer")
                    .unwrap_or(&cli_config.keypair_path),
            )?,
            authority: read_keypair_file(
                matches
                    .value_of("authority")
                    .unwrap_or(&cli_config.keypair_path)
            )?,
            verbose: matches.is_present("verbose"),
            dry_run: matches.is_present("dry_run"),
        }
    };
    solana_logger::setup_with_default("solana=info");
    let rpc_client =
        RpcClient::new_with_commitment(config.json_rpc_url.clone(), CommitmentConfig::confirmed());

    match (sub_command, sub_matches) {
        ("account", Some(arg_matches)) => {
            let name_record_address = pubkey_of(arg_matches, "name_record").unwrap();
            let name_record_header = get_name_record_account(&rpc_client, &name_record_address)?;
            
            println!("\nPublic Key: {}", name_record_address);
            println!("-------------------------------------------------------------------");

            println!("Name Record Data:\n    \
                    parent_name: {}\n    \
                    owner: {}\n    \
                    class: {}\n    \n",
                    name_record_header.parent_name,
                    name_record_header.owner,
                    name_record_header.class,
                );

            Ok(())
        }
        ("create-record", Some(arg_matches)) => {
            let name = value_t_or_exit!(arg_matches, "name", String);
            let owner_address = pubkey_of(arg_matches, "owner").unwrap_or(config.authority.pubkey());
            let lamports = value_t!(arg_matches, "lamports", u64).ok();
            let space = value_t!(arg_matches, "space", usize).ok();
            let parent_name_address = pubkey_of(arg_matches, "parent_name");

            let class_name = if arg_matches.is_present("class_name") {
                Some(
                    read_keypair_file(arg_matches.value_of("class_name").unwrap())?
                )
            } else { None };

            let parent_owner = if arg_matches.is_present("parent_owner") {
                Some(
                    read_keypair_file(arg_matches.value_of("parent_owner").unwrap())?
                )
            } else { None };
        
            process_create(
                &rpc_client,
                &config,
                &name,
                &owner_address,
                lamports,
                space,
                class_name,
                parent_name_address,
                parent_owner,
            )
        }
        ("wcreate-record", Some(arg_matches)) => {
            let name = value_t_or_exit!(arg_matches, "name", String);
            let owner_address = pubkey_of(arg_matches, "owner").unwrap_or(config.authority.pubkey());
            let lamports = value_t!(arg_matches, "lamports", u64).ok();
            let space = value_t!(arg_matches, "space", usize).ok();
            let parent_name_address = pubkey_of(arg_matches, "parent_name");

            let class_name = if arg_matches.is_present("class_name") {
                Some(
                    read_keypair_file(arg_matches.value_of("class_name").unwrap())?
                )
            } else { None };

            let parent_owner = if arg_matches.is_present("parent_owner") {
                Some(
                    read_keypair_file(arg_matches.value_of("parent_owner").unwrap())?
                )
            } else { None };

            process_wcreate(
                &rpc_client,
                &config,
                &name,
                &owner_address,
                lamports,
                space,
                class_name,
                parent_name_address,
                parent_owner,
            )
        }
        ("update-record", Some(arg_matches)) => {
            let name_record_address = pubkey_of(arg_matches, "name_record").unwrap();
            let data = value_t_or_exit!(arg_matches, "data", String);
            let data_bytes = bs58::decode(data).into_vec()?;
            let offset = value_t!(arg_matches, "offset", u32).unwrap_or(0);
            
            process_update_record(&rpc_client, &config, &name_record_address, &data_bytes, &offset)
        }
        ("transfer-ownership", Some(arg_matches)) => {
            let name_record_address = pubkey_of(arg_matches, "name_record").unwrap();
            let new_owner = pubkey_of(arg_matches, "new_owner").unwrap();
            let class_name = if arg_matches.is_present("class_account") {
                Some(
                    read_keypair_file(arg_matches.value_of("class_account").unwrap())?
                )
            } else { None };

            process_transfer_ownership_account(&rpc_client, &config, &name_record_address, &new_owner, class_name)
        }
        ("delete-record", Some(arg_matches)) => {
            let name_record_address = pubkey_of(arg_matches, "name_record").unwrap();
            let refund_address = pubkey_of(arg_matches, "refund_account").unwrap();

            process_delete_record(&rpc_client, &config, &name_record_address, &refund_address)
        }
        ("register-parent-name", Some(arg_matches)) => {
            let name = value_t_or_exit!(arg_matches, "name", String);
            let owner_address = pubkey_of(arg_matches, "owner").unwrap_or(config.authority.pubkey());
            let price = value_t_or_exit!(arg_matches, "price", u64);

            process_register_parent_name(&rpc_client, &config, &name, &owner_address, &price)
        }
        ("set-tln-issuer", Some(arg_matches)) => {
            let tln_issuer = pubkey_of(arg_matches, "tln_issuer").unwrap_or(config.authority.pubkey());

            // process_set_top_level_name_issuer(&rpc_client, &config, &tln_issuer)
            Ok(())
        }
        _ => unreachable!(),
    }
}

fn get_name_record_account(
    rpc_client: &RpcClient,
    name_record_address: &Pubkey,
) -> Result<NameRecordHeader, String> {
    let account = rpc_client
        .get_multiple_accounts(&[*name_record_address])
        .map_err(|err| err.to_string())?
        .into_iter()
        .next()
        .unwrap();

    match account {
        None => Err(format!(
            "Name record account {} does not exist",
            name_record_address
        )),
        Some(account) => NameRecordHeader::unpack_from_slice(&account.data).map_err(|err| {
            format!(
                "Failed to deserialize name record account {}: {}",
                name_record_address, err
            )
        }),
    }
}

fn get_name_record_address(
    name: &String,
    name_class: &Pubkey,
    parent_name_account: &Pubkey,
) -> Pubkey {

    let hashed_name = hashv(&[name.as_bytes()]);

    get_seeds_and_key(
        &spl_name_service::id(),
        hashed_name.as_ref().to_vec(),
        Some(name_class),
        Some(parent_name_account),
    ).0
}

fn get_min_required_lamports(
    rpc_client: &RpcClient,
    allocated_space: Option<usize>, 
) -> Result<u64, String> {
    let space = allocated_space.unwrap_or(0);
    rpc_client
        .get_minimum_balance_for_rent_exemption(NameRecordHeader::LEN + space)
        .map_err(|err| err.to_string())
}

fn process_create(
    rpc_client: &RpcClient,
    config: &Config,
    name: &String,
    owner_address: &Pubkey,
    lamports: Option<u64>,
    space: Option<usize>,
    class_name: Option<Keypair>,
    parent_name_address: Option<Pubkey>,
    parent_owner: Option<Keypair>,
) -> Result<(), Box<dyn std::error::Error>> {
    let min_required_lamports = get_min_required_lamports(&rpc_client, space)?;
    if lamports.is_some() && lamports.unwrap() < min_required_lamports {
        return Err(format!(
            "Lamports should be at least {}",
            min_required_lamports
        ).into());
    }

    let lamports = if lamports.is_some() {
        lamports.unwrap()
    } else { min_required_lamports };

    let balance = rpc_client.get_balance(&config.fee_payer.pubkey())?;
    if balance < lamports {
        return Err(format!(
            "Fee payer, {}, has insufficient balance: {} SOL required, {} SOL available",
            config.fee_payer.pubkey(),
            lamports_to_sol(lamports),
            lamports_to_sol(balance)
        )
        .into());
    }

    let is_class_name = class_name.is_some();
    let is_parent_owner = parent_owner.is_some();

    let hashed_name = hashv(&[name.as_bytes()]);
    let class_name_pubkey = if is_class_name { Some(class_name.as_ref().unwrap().pubkey()) } else { None };
    let parent_owner_pubkey = if is_parent_owner { Some(parent_owner.as_ref().unwrap().pubkey()) } else { None };

    let name_record_address = get_name_record_address(
        name,
        &class_name_pubkey.unwrap_or(Pubkey::default()), 
        &parent_owner_pubkey.unwrap_or(Pubkey::default())
    );


    if get_name_record_account(&rpc_client, &name_record_address).is_ok() {
        return Err(format!(
            "Name Record Account {} already exists",
            &name_record_address,
        )
        .into());
    }

    let mut create_record_transaction = Transaction::new_with_payer(
        &[
            spl_name_service::instruction::create(
                spl_name_service::id(),
                NameRegistryInstruction::Create{
                    hashed_name: hashed_name.as_ref().to_vec(),
                    lamports: lamports,
                    space: space.unwrap_or(NameRecordHeader::LEN) as u32,
                },
                name_record_address,
                config.fee_payer.pubkey(),
                *owner_address,
                class_name_pubkey,
                parent_name_address,
                parent_owner_pubkey,
            )?,
        ],
        Some(&config.fee_payer.pubkey()),
    );

    let blockhash = rpc_client.get_recent_blockhash()?.0;
    create_record_transaction.try_sign(&[&config.fee_payer], blockhash)?;
    
    if is_class_name {
        create_record_transaction.try_sign(&[&class_name.unwrap()], blockhash)?;
    }

    if is_parent_owner {
        create_record_transaction.try_sign(&[&parent_owner.unwrap()], blockhash)?;
    }

    send_transaction(&rpc_client, &config, create_record_transaction)?;

    println!("Name Record Account Address: {}", name_record_address);
    println!("Created!");
    Ok(())
}

fn process_wcreate(
    rpc_client: &RpcClient,
    config: &Config,
    name: &String,
    owner_address: &Pubkey,
    lamports: Option<u64>,
    space: Option<usize>,
    class_name: Option<Keypair>,
    parent_name_address: Option<Pubkey>,
    parent_owner: Option<Keypair>,
) -> Result<(), Box<dyn std::error::Error>> {
    let min_required_lamports = get_min_required_lamports(&rpc_client, space)?;
    if lamports.is_some() && lamports.unwrap() < min_required_lamports {
        return Err(format!(
            "Lamports should be at least {}",
            min_required_lamports
        ).into());
    }

    let lamports = if lamports.is_some() {
        lamports.unwrap()
    } else { min_required_lamports };

    let balance = rpc_client.get_balance(&config.fee_payer.pubkey())?;
    if balance < lamports {
        return Err(format!(
            "Fee payer, {}, has insufficient balance: {} SOL required, {} SOL available",
            config.fee_payer.pubkey(),
            lamports_to_sol(lamports),
            lamports_to_sol(balance)
        )
        .into());
    }

    let is_class_name = class_name.is_some();
    let is_parent_owner = parent_owner.is_some();

    let hashed_name = hashv(&[name.as_bytes()]);
    let class_name_pubkey = if is_class_name { Some(class_name.as_ref().unwrap().pubkey()) } else { None };
    let parent_owner_pubkey = if is_parent_owner { Some(parent_owner.as_ref().unwrap().pubkey()) } else { None };

    let name_record_address = get_name_record_address(
        name,
        &class_name_pubkey.unwrap_or(Pubkey::default()), 
        &parent_owner_pubkey.unwrap_or(Pubkey::default())
    );


    if get_name_record_account(&rpc_client, &name_record_address).is_ok() {
        return Err(format!(
            "Name Record Account {} already exists",
            &name_record_address,
        )
        .into());
    }

    // let mut create_record_transaction = Transaction::new_with_payer(
    //     &[
    //         vpl_name_register::instruction::wcreate(
    //             vpl_name_register::id(),
    //             vpl_name_register::instruction::NameRegisterInstruction::WCreate{
    //                 hashed_name: hashed_name.as_ref().to_vec(),
    //                 lamports: lamports,
    //                 space: space.unwrap_or(NameRecordHeader::LEN) as u32,
    //             },
    //             name_record_address,
    //             config.fee_payer.pubkey(),
    //             *owner_address,
    //             class_name_pubkey,
    //             parent_name_address,
    //             parent_owner_pubkey,
    //         )?,
    //     ],
    //     Some(&config.fee_payer.pubkey()),
    // );

    // let blockhash = rpc_client.get_recent_blockhash()?.0;
    // create_record_transaction.try_sign(&[&config.fee_payer], blockhash)?;
    
    // if is_class_name {
    //     create_record_transaction.try_sign(&[&class_name.unwrap()], blockhash)?;
    // }

    // if is_parent_owner {
    //     create_record_transaction.try_sign(&[&parent_owner.unwrap()], blockhash)?;
    // }

    // send_transaction(&rpc_client, &config, create_record_transaction)?;

    println!("Name Record Account Address: {}", name_record_address);
    println!("Created!");
    Ok(())
}

fn process_update_record(
    rpc_client: &RpcClient,
    config: &Config,
    name_record_address: &Pubkey,
    data: &Vec<u8>,
    offset: &u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let name_record_account = get_name_record_account(&rpc_client, &name_record_address)?;
    let required_authority = if name_record_account.class != Pubkey::default() {name_record_account.class} else {name_record_account.owner};

    if config.authority.pubkey() != required_authority {
        return Err(format!(
            "Authority mistmach in Name Record Account: {}\n \
            Provided authority: {}\n \
            Required authority: {}",
            &name_record_address,
            &config.authority.pubkey(),
            required_authority
        )
        .into());
    }

    let name_record_account_data = rpc_client
        .get_multiple_accounts(&[*name_record_address])
        .map_err(|err| err.to_string())?
        .into_iter()
        .next()
        .unwrap()
        .unwrap();

    if name_record_account_data.data.len() - NameRecordHeader::LEN <= data.len() {
        return Err(format!(
            "Invalid data length. Max allowed data length {}, to extend run first `extend-record-data-len`",
            name_record_account_data.data.len() - NameRecordHeader::LEN
        )
        .into())
    }

    let mut update_record_transaction = Transaction::new_with_payer(
        &[
            spl_name_service::instruction::update(
                spl_name_service::id(),
                *offset,
                data.to_vec(),
                *name_record_address,
                config.authority.pubkey(),
            )?,
        ],
        Some(&config.fee_payer.pubkey()),
    );

    let blockhash = rpc_client.get_recent_blockhash()?.0;
    update_record_transaction.try_sign(&[&config.fee_payer, &config.authority], blockhash)?;

    send_transaction(&rpc_client, &config, update_record_transaction)?;

    println!("Name Record {} data updated successful!", name_record_address);
    Ok(())
}

fn process_transfer_ownership_account(
    rpc_client: &RpcClient,
    config: &Config,
    name_record_address: &Pubkey,
    new_owner: &Pubkey,
    class_name: Option<Keypair>,
) -> Result<(), Box<dyn std::error::Error>> {
    let name_record_account = get_name_record_account(&rpc_client, &name_record_address)?;
    
    if config.authority.pubkey() != name_record_account.owner {
        return Err(format!(
            "Authority mistmach in Name Record Account: {}, \
            provided authority: {}, \
            required authority: {}",
            &name_record_address,
            &config.authority.pubkey(),
            name_record_account.owner,
        )
        .into());
    }

    let class_name_pubkey = if class_name.is_some() { Some((class_name.unwrap()).pubkey()) } else { None };
    let is_class_required = name_record_account.class != Pubkey::default();
    let is_signer_required_class = class_name_pubkey.is_some() && class_name_pubkey.unwrap() == name_record_account.class;
    if is_class_required && !is_signer_required_class {
        return Err(format!(
        "Required class-name argument, class-name not provided or invalid."
        )
        .into())
    }

    let mut transfer_ownership_transaction = Transaction::new_with_payer(
        &[
            spl_name_service::instruction::transfer(
                spl_name_service::id(),
                *new_owner,
                *name_record_address,
                config.authority.pubkey(),
                class_name_pubkey,
            )?,
        ],
        Some(&config.fee_payer.pubkey()),
    );

    let blockhash = rpc_client.get_recent_blockhash()?.0;
    transfer_ownership_transaction.try_sign(&[&config.fee_payer, &config.authority], blockhash)?;

    send_transaction(&rpc_client, &config, transfer_ownership_transaction)?;

    println!("Owner changed from {} to {} successful!", name_record_account.owner, new_owner);
    Ok(())
}

fn process_delete_record(
    rpc_client: &RpcClient,
    config: &Config,
    name_record_address: &Pubkey,
    refund_address: &Pubkey,
) -> Result<(), Box<dyn std::error::Error>> {
    let name_record_account = get_name_record_account(&rpc_client, &name_record_address)?;
    let refund_account = rpc_client
        .get_multiple_accounts(&[*refund_address])
        .map_err(|err| err.to_string())?
        .into_iter()
        .next()
        .unwrap();

    if refund_account.is_none() {
        return Err(format!(
            "Refund account must be already created. First create refund account {}.",
            refund_address,
        ).into())
    }
    
    if config.authority.pubkey() != name_record_account.owner {
        return Err(format!(
            "Authority mistmach in Name Record Account: {}, \
            provided authority: {}, \
            required authority: {}",
            &name_record_address,
            &config.authority.pubkey(),
            name_record_account.owner,
        )
        .into());
    }

    let mut delete_record_transaction = Transaction::new_with_payer(
        &[
            spl_name_service::instruction::delete(
                spl_name_service::id(),
                *name_record_address,
                config.authority.pubkey(),
                *refund_address,
            )?,
        ],
        Some(&config.fee_payer.pubkey()),
    );

    let blockhash = rpc_client.get_recent_blockhash()?.0;
    delete_record_transaction.try_sign(&[&config.fee_payer, &config.authority], blockhash)?;

    send_transaction(&rpc_client, &config, delete_record_transaction)?;

    println!("Name record deleted successful!");
    Ok(())
}

fn process_register_parent_name(
    rpc_client: &RpcClient,
    config: &Config,
    name: &String,
    owner_address: &Pubkey,
    price: &u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let lamports = get_min_required_lamports(&rpc_client, None)?;

    let balance = rpc_client.get_balance(&config.fee_payer.pubkey())?;
    if balance < lamports {
        return Err(format!(
            "Fee payer, {}, has insufficient balance: {} SOL required, {} SOL available",
            config.fee_payer.pubkey(),
            lamports_to_sol(lamports),
            lamports_to_sol(balance)
        )
        .into());
    }

    let hashed_name = hashv(&[name.as_bytes()]);

    // let (name_registry_class_address, _) = Pubkey::find_program_address(
    //     &[&vpl_name_register::processor::SUFFIX_PARENT_NAME_CLASS], 
    //     &vpl_name_register::id(),
    // );

    // let name_record_address = get_name_record_address(
    //     name,
    //     &name_registry_class_address, 
    //     &Pubkey::default(),
    // );

    // if get_name_record_account(&rpc_client, &name_record_address).is_ok() {
    //     return Err(format!(
    //         "Name Record Account {} already exists",
    //         &name_record_address,
    //     )
    //     .into());
    // }

    // let (parent_name_address, parent_name_bump_seed_nonce) = Pubkey::find_program_address(
    //     &[
    //         &name_record_address.to_bytes()[..32],
    //         &hashed_name.as_ref().to_vec()[..32],
    //     ], 
    //     &vpl_name_register::id()
    // );

    // let mut register_parent_name_transaction = Transaction::new_with_payer(
    //     &[
    //         register_parent_name(
    //             NameRegisterInstruction::RegisterParentName{
    //                 name: name.as_bytes().to_vec(),
    //                 price: *price,
    //                 name_register_seed: parent_name_bump_seed_nonce,
    //         },
    //         parent_name_address,
    //         name_registry_class_address,
    //         name_record_address,
    //         vpl_name_register::processor::find_name_account_owner(&owner_address).0,
    //         *owner_address,
    //         config.fee_payer.pubkey(),
    //     )
    //     ],
    //     Some(&config.fee_payer.pubkey()),
    // );

    // let blockhash = rpc_client.get_recent_blockhash()?.0;
    // register_parent_name_transaction.try_sign(&[&config.fee_payer, &config.authority], blockhash)?;

    // send_transaction(&rpc_client, &config, register_parent_name_transaction)?;

    println!("Register parent name successful!");
    Ok(())
}

// fn process_set_top_level_name_issuer(
//     rpc_client: &RpcClient,
//     config: &Config,
//     tln_issuer: &Pubkey,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let lamports = get_min_required_lamports(&rpc_client, None)?;

//     let balance = rpc_client.get_balance(&config.fee_payer.pubkey())?;
//     if balance < lamports {
//         return Err(format!(
//             "Fee payer, {}, has insufficient balance: {} SOL required, {} SOL available",
//             config.fee_payer.pubkey(),
//             lamports_to_sol(lamports),
//             lamports_to_sol(balance)
//         )
//         .into());
//     }

//     let mut set_tpl_issuer = Transaction::new_with_payer(
//         &[
//             set_top_level_name_issuer(
//                 vpl_name_register::id(), 
//                 NameIssuerInstruction::SetIssuerTLN, 
//                 *tln_issuer, 
//                 config.fee_payer.pubkey())
//         ],
//         Some(&config.fee_payer.pubkey()),
//     );

//     let blockhash = rpc_client.get_recent_blockhash()?.0;
//     set_tpl_issuer.try_sign(&[&config.fee_payer, &config.authority], blockhash)?;

//     send_transaction(&rpc_client, &config, set_tpl_issuer)?;

//     println!("Register parent name successful!");
//     Ok(())
// }

pub fn is_base58(bs58_str: String) -> Result<(), String>
{   
    if bs58::decode(bs58_str.clone()).into_vec().is_ok() {
        Ok(())
    } else {
        Err(format!(
            "Unable to parse data in base58 format: `{}`",
            &bs58_str
        ))
    }
}

pub fn is_valid_account_name(name: String) -> Result<(), String>
{
    let is_valid_name = name.chars().any(|c| c.is_ascii());
    if is_valid_name {
        Ok(())
    } else {
        Err(format!(
            "Invalid `account-name`: `{}`",
            &name
        ))
    }
}
