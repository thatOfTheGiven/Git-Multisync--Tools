/* Git-lfs-Sync::inspect
 *============================================
 * Purpose:     Inspect the export
 */

use std::env;
use std::{fs, io};
use std::io::{Read, Write};
use std::fs::File;
use std::path::{Path, PathBuf};

use base64::Engine;
use base64::prelude::BASE64_STANDARD;

use serde_json::{Value};

use std::collections::HashMap;

use substring::Substring;

use clap::ArgMatches;



/* run
 *============================================
 * Purpose:     run an inspect
 * Input:       ArgMAtches
 * Results:     NONE
 * Notes:       
 */
pub fn run (sub: &ArgMatches)
{
    let inspect: &ArgMatches;

    if sub.subcommand_matches("Content").is_some()
    {inspect = sub.subcommand_matches("Content").unwrap();}
    else if sub.subcommand_matches("Json").is_some()
    {inspect = sub.subcommand_matches("Json").unwrap();}
    else if sub.subcommand_matches("Files").is_some()
    {inspect = sub.subcommand_matches("Files").unwrap();}
    else 
    {inspect = sub.subcommand_matches("Hash").unwrap();}


    let input: &str = inspect.get_one::<String>("import").unwrap();
    

    
    if sub.subcommand_matches("Content").is_some()
    {
        let mut out: Option<&Path> = None;

        if let Some(out_file)  = inspect.get_one::<String>("outfile")
        {out = Some(Path::new(out_file));}


        extract_content(input, inspect.get_one::<bool>("build_structure").unwrap(), out);
    }
    else 
    {
        let content: Vec<u8> = get_json(input);  //json and pack content
    
        if sub.subcommand_matches("Json").is_some()
        {
            if let Some(out) = inspect.get_one::<String>("outfile")
            {
                let out_file = define_file_name(input, out, "json");
                fs::write(out_file, std::str::from_utf8(&content).unwrap()).expect("inspect, run: Unable to write file");
            }
            else {io::stdout().write_all(&content).unwrap();}
        }
        else 
        {
            let json_string: String =String::from_utf8(content).unwrap();
            let json: Value = serde_json::from_str(&json_string).unwrap();

            let mut data: String = "".to_string();
            let format: String   = inspect.get_one::<String>("format").unwrap().to_string();

            let extention: String;

            let mut mapping: HashMap<String, Vec<String>> = HashMap::new();

            let json_mapping = json["mapping"].as_object().unwrap();

            for hash in json_mapping.keys()
            {
                let key: String;
                let value: String;
                if sub.subcommand_matches("Files").is_some()
                {
                    key   = json_mapping[hash].as_str().unwrap().to_string();
                    value = hash.to_string();
                }
                else  
                {
                    key   = hash.to_string();
                    value = json_mapping[hash].as_str().unwrap().to_string();
                }    

                if !mapping.contains_key(&key)
                {mapping.insert(key.clone(), vec![]);}

                mapping.get_mut(&key).unwrap().push(value);
            }

            let mut keys: Vec<&String> = mapping.keys().collect();
            keys.sort();


            if sub.subcommand_matches("Files").is_some()
            {
                extention = "fl".to_string();
            }                
            else 
            {
                extention = "hs".to_string();
            }


            for key in keys
            {
                if data != ""  {data += "\n";}


                if format == "Long"
                {
                    let mut first_sub = true;
                    for value in &mapping[key].clone()
                    {
                        if !first_sub {data += "\n";}
                        data += &(key.to_owned() + " - " + &value);

                        first_sub = false;
                    }
                }
                else{data += key}
            }


            if let Some(out) = inspect.get_one::<String>("outfile")
            {
                let out_file = define_file_name(input, out, &extention);
                fs::write(out_file, data.clone()).expect("inspect, run: Unable to write file");
            }
            else {print!("{}", data);}
        }
    }
}

/* get_json
 *============================================
 * Purpose:     split the import into pack and json
 * Input:       import file
 * Results:     json and pack file
 * Notes:       
 */
pub fn get_json(input: &str) -> Vec<u8>
{
    let sync_pack: &Path = Path::new(input);
    
    let mut obj  = File::open(sync_pack).unwrap();
    let mut data = [0; 10];
    let mut json_bit: Vec<u8> = vec![];
    
    let mut size = 1;
    while size > 0
    {
        size = obj.read(&mut data).unwrap();

        for bit in &data
        {
             if *bit == b'\n'    
            {return json_bit}
            json_bit.push(*bit);
        }
    } 

    return json_bit
}

/* extract_content
 *============================================
 * Purpose:     split the import into pack and json
 * Input:       import file
 * Results:     json and pack file
 * Notes:       
 */
pub fn extract_content(input: &str, build_str: &bool, root: Option<&Path>)
{
    let sync_pack: &Path = Path::new(input);
    
    let mut obj  = File::open(sync_pack).unwrap();
    let mut data =  [0; 10];
    
    let mut json: bool = true;
    let mut is_data: bool = false;

    let mut encoded: Vec<u8> = vec![];
    let mut id: Vec<u8> = vec![]; 

    let mut size = 1;  
    while  size > 0
    {  
        size = obj.read(&mut data).unwrap();

        for bit in &data[..size]
        {            
            if *bit == b'\n'   
            {
                if !json
                {
                    write_obj(&String::from_utf8(id).unwrap(), &encoded, build_str, root);

                    encoded = vec![];
                    id = vec![];   
                }

                json = false;
                is_data = false;
            }
            else if !json
            {
                if !is_data
                {
                    if *bit != b':' {id.push(*bit)}
                    else {is_data = true}
                }
                else {encoded.push(*bit)}
            }
        }
    } 

    write_obj(&String::from_utf8(id).unwrap(), &encoded, build_str, root);
}


/* write_pack
 *============================================
 * Purpose:     write a pack file either to file or stdout
 * Input:       pack contents, out file or None
 * Results:     NONE
 * Notes:       
 */
fn write_obj(name: &str, encoded: &Vec<u8>, build_str: &bool, root: Option<&Path>)
{
    let  mut obj_path: PathBuf;
    if root != None {obj_path = root.unwrap().to_path_buf();}
    else            {obj_path = env::current_dir().unwrap();}

    if *build_str
    {
        obj_path = obj_path.join(name.substring(0, 2)).join(name.substring(2, 4));
        if !obj_path.exists()
        { let _ = std::fs::create_dir_all(obj_path.clone());}
    }

    let mut obj_file = File::create(obj_path.join(name)).unwrap();
    obj_file.write(&BASE64_STANDARD.decode(encoded).unwrap()).unwrap();
}

/* define_file_name
 *============================================
 * Purpose:     produce a file name
 * Input:       sync pack, out dir, and extension
 * Results:     Path
 * Notes:       
 */
fn define_file_name(sync_pack: &str, out: &str, extension: &str) -> PathBuf
{

    let mut path = PathBuf::from(out);
    if path.is_file()
    {panic!("inspect, define_file_name: {:?} does exists and is a file", path.to_str());}
    else if path.is_dir()
    {
        let mut file_name = sync_pack.substring(sync_pack.rfind("/").unwrap()+1, sync_pack.len()).to_string();

        file_name = file_name.substring(0, file_name.find(".").unwrap()).to_string();

        path.push(file_name + "." + extension);
    }

    return path
}
