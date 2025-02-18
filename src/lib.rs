mod programs;

#[cfg(test)] 
mod tests {

    use anchor_client::{
        Client, Cluster,
    };
    use bs58;
    use crate::programs::turbine_prereq::{
        turbine_prereq::ID,
        turbine_prereq::client,
    };
    use solana_client::rpc_client::RpcClient;
    use solana_program::{
        pubkey::Pubkey,
        system_instruction::transfer,
        system_program
    };
    use solana_sdk::{
        message::Message, signature::{read_keypair_file, Keypair, Signer}, transaction::Transaction
    };
    use std::{
        io::{self, BufRead},
        rc::Rc,
        str::FromStr,
    };
    
    const RPC_URL: &str = "https://api.devnet.solana.com";

    #[test]
    fn keygen() {
        // Create a new keypair
        let kp = Keypair::new();
        println!("You've generated a new Solana wallet: {}", kp.pubkey().to_string()); 
        println!(""); 
        println!("To save your wallet, copy and paste the following into a JSON file:");
        println!("{:?}", kp.to_bytes());
    } 
    
    #[test]
    fn base58_to_wallet() {
        println!("Input your private key as base58:");
        let stdin = io::stdin();
        let base58 = stdin.lock()
            .lines()
            .next()
            .unwrap()
            .unwrap(); 
        println!("Your wallet file is:"); 
        let wallet = bs58::decode(base58)
            .into_vec()
            .unwrap(); 
        println!("{:?}", wallet);
    }
    
    #[test]
    fn wallet_to_base58() {
        println!("Input your private key as a wallet file byte array:"); 
        let stdin = io::stdin(); 
        let wallet = stdin.lock().lines().next()
            .unwrap().unwrap()
            .trim_start_matches('[')
            .trim_end_matches(']')
            .split(',').map(|s| 
                s.trim()
                .parse::<u8>()
                .unwrap()
            ).collect::<Vec<u8>>();
        println!("Your private key is:");
        let base58 = bs58::encode(wallet).into_string();
        println!("{:?}", base58);
    }

    #[test] 
    fn airdop() {
        // Read the wallet file
        let keypair = read_keypair_file("wallets/dev-wallet.json")
            .expect("Couldn't find wallet file");
        // Connected to Solana Devnet RPC Client
        let client = RpcClient::new(RPC_URL);
        // Request airdrop
        match client.request_airdrop(&keypair.pubkey(), 2_000_000_000u64) 
        {
            Ok(s) => {
                println!("Success! Check out your TX here:");
                println!("https://explorer.solana.com/tx/{}?cluster=devnet", s.to_string());
            },
            Err(e) => {
                println!("Oops, something went wrong: {}", e.to_string())
            }
        };
    }
    
    #[test]
    fn transfer_sol() {
        // Import our keypair
        let keypair = read_keypair_file("wallets/dev-wallet.json")
            .expect("Couldn't find wallet file");
        // Define our Turbin3 public key
        let to_pubkey = Pubkey::from_str("2TyWe2XmL5ZMDyM69Rqnrnspum1X95CMGX2DWNaDmHmm").unwrap();
        // Create a Solana devnet connection
        let rpc_client = RpcClient::new(RPC_URL);
        // Get recent blockhash
        let recent_blockhash = rpc_client.get_latest_blockhash()
            .expect("Failed to get recent blockhash");
        // Get balance of dev wallet
        let balance = rpc_client
            .get_balance(&keypair.pubkey())
            .expect("Failed to get balance");
        // Create a test transaction to calculate fees
        let message = Message::new_with_blockhash(
            &[transfer( &keypair.pubkey(), &to_pubkey, balance)], 
            Some(&keypair.pubkey()), 
            &recent_blockhash
        );
        // Calculate exact fee rate to transfer entire SOL amount out of account minus fees 
        let fee = rpc_client
            .get_fee_for_message(&message) 
            .expect("Failed to get fee calculator");
        // Deduct fee from lamports amount and create a TX with correct balance 
        let transaction = Transaction::new_signed_with_payer(
            &[transfer(&keypair.pubkey(), &to_pubkey, balance - fee)], 
            Some(&keypair.pubkey()), 
            &vec![&keypair],
            recent_blockhash
        );
        // Send the transaction
        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send transaction");
        // Print our transaction out 
        println!("Success! Check out your TX here: https://explorer.solana.com/tx/{}/?cluster=devnet", signature);
    }

    #[test]
    fn register() -> Result<(), Box<dyn std::error::Error>> {
        
        // Let's define our accounts
        let payer = read_keypair_file("wallets/turbin3-wallet.json")
            .expect("Couldn't find wallet file");
        
        // Create a Solana devnet connection
        let client = Client::new(Cluster::Devnet, Rc::new(&payer));
        
        // Create program
        let program = client.program(ID)?;

        // Create the PDA for the transaction
        let (prereq, _bump) = Pubkey::find_program_address(
            &[b"prereq", payer.pubkey().as_ref()],
            &ID
        );

        // Define transaction accounts and arguments
        let complete_accounts = client::accounts::Complete {
            signer: payer.pubkey(),
            prereq: prereq, 
            system_program: system_program::id()
        };
        let complete_args = client::args::Complete {
            github: b"0xSatoriSync".to_vec(),
        };

        // Send the TX
        let signature = program
           .request()
           .accounts(complete_accounts)
           .args(complete_args)
           .signer(&payer)
           .send()?;

        // Print our transaction out 
        println!("Success! Check out your TX here: https://explorer.solana.com/tx/{}/?cluster=devnet", signature);
        Ok(())
        
    }

    #[test]
    fn update() -> Result<(), Box<dyn std::error::Error>> {
        
        // Let's define our accounts
        let payer = read_keypair_file("wallets/turbin3-wallet.json")
            .expect("Couldn't find wallet file");
        
        // Create a Solana devnet connection
        let client = Client::new(Cluster::Devnet, Rc::new(&payer));
        
        // Create program
        let program = client.program(ID)?;

        // Create the PDA for the transaction
        let (prereq, _bump) = Pubkey::find_program_address(
            &[b"prereq", payer.pubkey().as_ref()],
            &ID
        );

        // Define transaction accounts and arguments
        let update_accounts = client::accounts::Update {
            signer: payer.pubkey(),
            prereq: prereq, 
            system_program: system_program::id()
        };
        let update_args = client::args::Update {
            github: b"0xSatoriSync".to_vec(),
        };

        // Send the TX
        let signature = program
           .request()
           .accounts(update_accounts)
           .args(update_args)
           .signer(&payer)
           .send()?;

        // Print our transaction out 
        println!("Success! Check out your TX here: https://explorer.solana.com/tx/{}/?cluster=devnet", signature);
        Ok(())

    }

}
