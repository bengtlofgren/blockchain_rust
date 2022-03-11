use std::error::Error;
use sha2::Sha256;
use sha2::Digest;
use serde::{Deserialize, Serialize};
// use serde_json::Result;
// use serde_json::json;

#[derive(Serialize, Deserialize)]
struct Input {
    id : u64,
    amount : u64,
}
#[derive(Serialize, Deserialize)]
struct Transaction {
    inputs : Vec<Input>,
    outputs : Vec<Input>,
}

#[derive(Serialize, Deserialize)]
struct Block {
    hash : String,
    predecessor : String,
    nonce : u32,
    difficulty : u32,
    transactions : Vec<Transaction>,
}

#[derive(Serialize, Deserialize)]
struct State {
    height : u64,
    totalwork : u128,
    hash : String,
    outputs : Vec<Input>,
}

#[derive(Serialize, Deserialize)]
struct Head {
    height : u64,
    totalwork : u128,
    hash : String,
}

#[derive(Serialize, Deserialize)]
struct Blockchain {
    state : State,
    heads : Vec<Head>,
    blocks : Vec<Block>,
    head_history : Vec<Head>,
}

impl Blockchain {
    fn update_state(&mut self, predecessor : String, new_hash : String, diff : u32) {
        let work = u128::pow(16, diff);
        // check so this doesnt break things
        let mut new_head = Head{
            hash : new_hash.to_string(),
            totalwork : work,
            height : 1
        };
        for head in self.head_history.iter(){
            if predecessor.eq(&head.hash.to_string()) {
                new_head.totalwork += head.totalwork;
                new_head.height += head.height;
                
                if new_head.height >= self.state.height {
                    self.state.height = new_head.height;
                    self.state.totalwork = new_head.totalwork;
                    self.state.hash = new_head.hash.to_string();
                }
            }
        }
        let mut head_updated = false;
        for head in self.heads.iter_mut(){
            if predecessor.eq(&head.hash.to_string()) {
                head.hash = new_head.hash.to_string();
                head.height = &new_head.height + 0;
                head.totalwork = &new_head.totalwork + 0;
                head_updated = true;
            };
        }
        let head_copy = Head{
            hash : new_head.hash.to_string(),
            height : &new_head.height + 0,
            totalwork : &new_head.totalwork + 0
        };
        if !head_updated {
            self.heads.push(head_copy);
        }
        self.head_history.push(new_head);
        

    }

    fn add_block(&mut self, block : Block) {
        let diff = block.difficulty + 0;
        let blockhash = block.hash.to_string();
        let predecessor = block.predecessor.to_string();
        let transactions = serde_json::to_value(&block.transactions).unwrap();
        self.update_state(predecessor, blockhash, diff);
        self.update_outputs(transactions);
    }

    fn update_outputs(&mut self, trans : serde_json::Value){
        let transactions = trans[0];
        let outputs = transactions.get("outputs");
        match outputs{
            None => {},
            Some(outputs) => {
                if outputs.is_array(){
                    let ous = outputs.as_array().unwrap();
                    self.state.outputs.append(&mut ous);
                };
            }
        }
        let inputs = transactions.get("inputs");
        match inputs{
            None => {},
            Some(inputs) => {
                if inputs.is_array(){
                    let ins = inputs.as_array();
                    for i in ins.iter(){
                    self.state.outputs.retain(|x| *x.id == i.id);
                    };
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {  
    let cur_state = State{
        height : 0,
        totalwork : 0,
        hash : "".to_string(),
        outputs : vec![init_input(0,0),],
    };

    let cur_trans = Transaction{
        inputs : vec![init_input(0,0),],
        outputs : vec![init_input(0,0),],
    };
    
    let cur_block = Block{
        hash : "".to_string(),
        predecessor : "".to_string(),
        nonce : 0,
        difficulty : 0,
        transactions : vec![cur_trans,],
    };

    let cur_head = Head{
        height : 0,
        totalwork : 0,
        hash : "".to_string(),
    };

    let cur_head2 = Head{
        height : 0,
        totalwork : 0,
        hash : "".to_string(),
    };

    let mut blockchain = Blockchain{
        state : cur_state,
        heads: vec![cur_head,],
        blocks : vec![cur_block,],
        head_history : vec![cur_head2],
    };
    let mut is_init = false;
    loop {
        let stdin = std::io::stdin();
        let stdin = stdin.lock();
        let deserializer = serde_json::Deserializer::from_reader(stdin);
        let iterator = deserializer.into_iter::<serde_json::Value>();
        for item in iterator {
            match item {
                Ok(_) => {},
                Err(_) => {println!("Error : Init Error"); break;},
            };
            let json = item.unwrap();
            let k = json.get("init");
            match k {
                None => {
                    if !is_init{
                    println!("Error : Init Error "); break;}
                    else{
                    }
                },
                Some(k) => {
                    if is_init{
                        println!("Error: Already Initialised"); break;}
                    else{
                    is_init = validate_init(&k);
                    if is_init{
                        let new_block : Block = serde_json::from_value(k.to_owned()).unwrap();
                        blockchain.add_block(new_block);
                    };
                    continue;}
                },
            }
            let query = json.get("query");
            match query {
                None => {},
                Some(q) => {
                    if q.eq(&"state".to_string()) {
                        if is_init{
                            state_query(&blockchain); continue;}
                            else{
                            println!("Cannot query an uninitialised blockchain");
                            continue;}
                    }
                    else if q.eq(&"heads".to_string()) {
                        if is_init{
                            heads_query(&blockchain); continue;}
                    }
                    else{
                        println!("invalid query");
                        break;
                    }
                },
            }
            let block_sub = json.get("block");
            match block_sub {
                None => {
                    if !is_init{
                    println!("Error : Must initialise first"); break;}
                    else{
                    }
                },
                Some(b) => {
                    if is_init{
                        handle_block(&b, &mut blockchain); break; }
                    else{
                        println!("Error : Must initialise first"); break;
                    }
                },
            }

        };
    };
}


fn validate_init(json : &serde_json::Value) -> bool{
    let mut is_correct = true;
    match json.get("difficulty") {
        None => {
            println!("could not find _ field");
            is_correct = false;
        },
        Some(k) => {
            if k != 0 {
                println!("expected difficulty field to be 0");
                is_correct = false;
            };
        }
    }
    match json.get("nonce") {
        None => {
            println!("could not find nonce field");
            is_correct = false;
        },
        Some(k) => {
            if k != 0 {
                println!("expected nonce field to be 0");
                is_correct = false;
            };
        }
    }
    match json.get("predecessor") {
        None => {
            println!("could not find predecessor field");
            is_correct = false;
        },
        Some(k) => {
            if k != "" {
                println!("expected predecessor field to be empty string");
                is_correct = false;
            };
        }
    }
    match json.get("transactions") {
        None => {
            println!("could not find transactions field");
            is_correct = false;
        },
        Some(k) => {
            if !k.is_array(){
                println!("expected an array for transactions field")
            };
            }
        }
    
        if is_correct{
            is_correct = verify_hash(&json);
        }
        is_correct
}
fn verify_hash(json : &serde_json::Value) -> bool{
    
    let mut is_correct = true;
    let pred_str = &json.get("predecessor").unwrap().to_string();
    let trans_str = &json.get("transactions").unwrap().to_string();
    let diff_str = &json.get("difficulty").unwrap().to_string();
    let nonce_str = &json.get("nonce").unwrap().to_string();
    let encoded = format!("[{},{},{},{}]", pred_str, trans_str, diff_str, nonce_str);
    let mut sha256 = Sha256::new();
    sha256.update(encoded);
    let block_hash: String = format!("0x{:X}", sha256.finalize());
    // One string has double quotes whereas the other does not, so I do some hacky manipulation here, ideally would prefer not to ...
    let check_aux = format!("{:?}", block_hash.to_lowercase());
    if check_aux.eq(&json.get("hash").unwrap().to_string()) {
        println!("hash verified")
    }
    else{
        println!("hash invalid!");
        is_correct = false;
    }
    is_correct
}
fn state_query(blockchain : &Blockchain){
    let test_str = serde_json::to_string(&blockchain.state).unwrap();
    println!("{}", test_str);
}
fn heads_query(blockchain : &Blockchain){
    let test_str = serde_json::to_string(&blockchain.heads).unwrap();
    println!("{}", test_str);
}

fn handle_block(json : &serde_json::Value, blockchain : &mut Blockchain){
    println!("handling block");
    let mut is_correct = false;
    if validate_block(json) {
        let mut new_block : Block = serde_json::from_value(json.to_owned()).unwrap();
        let blockhash = &new_block.hash;
        let blockpred = &new_block.predecessor;
        //Check if there is a predecessor
        for pred in blockchain.blocks.iter_mut() {
            //I couldnt get match to work here ...
            if &pred.hash == blockhash {
                println!("duplicate hash");
                return;
            };
            if &pred.hash == blockpred {
                is_correct = validate_difficulty(&pred.difficulty, &new_block.difficulty, blockhash);
            };
        };
        if !is_correct{
            println!("no predecessor found");
            return;
        };
        if is_correct {
            is_correct = validate_transactions(&mut new_block.transactions);
            if !is_correct{
                println!("invalid transaction");
            };
        };
        if is_correct {
            blockchain.add_block(new_block);
        };
    };
}

fn init_input(id : u64, amount : u64) -> Input{
    Input {
        id,
        amount,
    }
}

fn validate_block(json : &serde_json::Value) -> bool{
    let mut is_correct = true;
    match json.get("difficulty") {
        None => {
            println!("could not find _ field");
            is_correct = false;
        },
        Some(k) => {
            if !k.is_u64() {
                println!("expected difficulty field to >=0");
                is_correct = false;
            };
        }
    }
    match json.get("nonce") {
        None => {
            println!("could not find nonce field");
            is_correct = false;
        },
        Some(k) => {
            if !k.is_u64() {
                println!("expected nonce field to be >= 0");
                is_correct = false;
            };
        }
    }
    match json.get("predecessor") {
        None => {
            println!("could not find predecessor field");
            is_correct = false;
        },
        //Might want to do some check here that it is a string but I don't know how to do that. k.is_string?
        Some(_k) => {}
    }
    match json.get("transactions") {
        None => {
            println!("could not find transactions field");
            is_correct = false;
        },
        Some(k) => {
            if !k.is_array(){
                println!("expected an array for transactions field")
            };
            }
        }
    
        if is_correct{
            is_correct = verify_hash(&json);
        }
        is_correct
}
fn validate_difficulty(pred_diff : &u32, new_diff : &u32, blockhash :&String) -> bool{
    println!("validating difficulty");
    let mut is_correct = true;
    if pred_diff > new_diff {
        is_correct = false;
        println!("difficulty must not decrease");
        return is_correct;
    }
    let mut zero_count = 0;
    for (i,c) in blockhash.chars().enumerate() {
        if i <= 1 {
        }
        else{
            if c == '0' {
                zero_count += 1;
            }
            else{
                break;
            }
        }
    }
    if &zero_count < pred_diff {
        is_correct = false;
        println!("leading zeroes in block hash did not match difficulty")
    }
    is_correct
}
fn validate_transactions(transactions : &mut Vec<Transaction>) -> bool {
    println!("validating transactions!");
    let mut is_correct = true;
    let mut outputs_sum = 0;
    let mut inputs_sum = 0;
    for trans in transactions.iter_mut() {
        let json = serde_json::to_value(trans).unwrap();
        let inputs = json.get("inputs");
        match inputs {
            None => {
            },
            Some(ins) => {
                if !ins.is_array(){
                    is_correct = false;
                    return is_correct;
                }
                else{
                    let input_sum = sum_inputs(ins);
                    if input_sum <= 0 {
                        is_correct = false;
                        return is_correct;
                    }
                    else{
                        inputs_sum += input_sum;
                    }
                }
            }
        }
        let outputs = json.get("outputs");
        match outputs {
            None => {
            },
            Some(ous) => {
                if !ous.is_array(){
                    println!("expected an array of outputs");
                    is_correct = false;
                    return is_correct;
                }
                else{
                let output_sum = sum_inputs(ous);
                if output_sum <= 0 {
                    is_correct = false;
                    return is_correct;
                }
                else{
                    outputs_sum += output_sum;
                }
                }
            }
        }
        is_correct = inputs_sum == outputs_sum;
    }
    is_correct
}

fn sum_inputs(json : &serde_json::Value) -> i64 {
    let mut total_amount = 0;
    let inputs = json.as_array().unwrap();
    for inp in inputs.iter(){
        //inp is a serde_json::Value
        let amount = inp.get("amount");
        match amount {
            None => {
            },
            Some(a) => {
                //unsafe
                if !a.is_u64(){
                    println!("amount is not u64");
                    return -1;
                };

                total_amount += a.as_u64().unwrap();
                }
            }
        }
    total_amount.try_into().unwrap()
}

