/* Git-Pack-Sync::inspect
 *============================================
 * Purpose:     Inspect the export
 */

use std::{fs, io};
use std::io::{Read, Write};
use std::fs::File;
use std::path::{Path, PathBuf};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use serde_json::{Value, json};
use std::collections::HashMap;
use substring::Substring;
use clap::ArgMatches;



pub struct ObjRef {
    current: serde_json::Value,
    previous: serde_json::Value,
}



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

    if sub.subcommand_matches("Pack").is_some()
    {inspect = sub.subcommand_matches("Pack").unwrap();}
    else if sub.subcommand_matches("Json").is_some()
    {inspect = sub.subcommand_matches("Json").unwrap();}
    else if sub.subcommand_matches("Branches").is_some()
    {inspect = sub.subcommand_matches("Branches").unwrap();}
    else // Tags
    {inspect = sub.subcommand_matches("Tags").unwrap();}


    let input: &str = inspect.get_one::<String>("import").unwrap();
    let content: (Vec<u8>, Vec<u8>) = split_import(input);  //json and pack content

    
    if sub.subcommand_matches("Pack").is_some()
    {
        if content.1.len()>4
        {
            let mut out_path: Option<PathBuf> = None;
            if let Some(out) = inspect.get_one::<String>("outfile")
            {out_path =  Some(define_file_name(input, out, "pack"));}
            
            write_pack(&content.1, &out_path);
        }
        else  {println!("No Pack content found")}
    }
    else if sub.subcommand_matches("Json").is_some()
    {
        if let Some(out) = inspect.get_one::<String>("outfile")
        {
            let out_file = define_file_name(input, out, "json");
            fs::write(out_file, std::str::from_utf8(&content.0).unwrap()).expect("inspect, run: Unable to write file");
        }
        else {io::stdout().write_all(&content.0).unwrap();}
    }
    else 
    {
        let json_string: String =String::from_utf8(content.0).unwrap();
        let json: Value = serde_json::from_str(&json_string).unwrap();

        let mut data: String = "".to_string();
        let status: String   = inspect.get_one::<String>("status").unwrap().to_string();
        let format: String   = inspect.get_one::<String>("format").unwrap().to_string();

        let extention: String;


        if sub.subcommand_matches("Branches").is_some()
        {
            extention = "br".to_string();
            let mut branches: HashMap<String, ObjRef> = HashMap::new();

            let json_branches = json["branch"].as_object().unwrap();
            let json_active = json_branches["active"].as_object().unwrap();
            if !json_active.is_empty() && (status == "Active" || status == "All")
            {
                for branch in json_branches["active"].as_object().unwrap().keys()
                {
                    let json_branch = &json_active[branch];
                    branches.insert(branch.to_string(),  ObjRef {current: json_branch["current"].clone(), previous: json_branch["previous"].clone()});
                }
            }

            let json_retired = json_branches["retired"].as_object().unwrap();
            if !json_retired.is_empty()  && (status == "Retired" || status == "All")
            {
                for branch in json_retired.keys()
                {branches.insert(branch.to_string(),  ObjRef {current: json!(null), previous: json_retired[branch].clone()});}
            }

            let mut keys: Vec<&String> = branches.keys().collect();
            keys.sort();

            for branch in keys
            {
                if data != ""  {data += "\n";}
                

                if format == "Event"
                {
                    if branches[branch].previous.is_null()     {data += "C\t";}
                    else if branches[branch].current.is_null() {data += "R\t";}
                    else                                       {data += "M\t";}
                }

                data += &branch;


                if format == "Index"
                {
                    if !branches[branch].current.is_null()
                    {data  += &("\t".to_owned() + &branches[branch].current.as_str().unwrap());}
                }
                else if format == "Long"
                {
                    data += "\t";

                    
                    if !branches[branch].previous.is_null()
                    {data += &branches[branch].previous.as_str().unwrap();}
                    data += ":";
                    if !branches[branch].current.is_null()
                    {data +=  &branches[branch].current.as_str().unwrap();}
                }
            }
        }
        else // "Tag"
        {
            extention = "tg".to_string();
            let tag_type: String = inspect.get_one::<String>("type").unwrap().to_string();


            let mut tags: HashMap<String, ObjRef> = HashMap::new();

            let json_tags      = json["tag"].as_object().unwrap();
            let json_light     = json_tags["light"].as_object().unwrap();
            let json_annotated = json_tags["annotated"].as_object().unwrap();
            let json_retired   = json_tags["retired"].as_object().unwrap();

            if !json_light.is_empty() && (status == "Active" || status == "All") && (tag_type == "Light" || tag_type == "All")
            {
                for tag in json_light.keys()
                {
                    let json_tag = json_light[tag].as_object().unwrap();
                    tags.insert(tag.to_string(),  ObjRef {current: json_tag["current"].clone(), previous: json_tag["previous"].clone()});
                }
            }

            println!("{:?}", json_annotated);

            if !json_annotated.is_empty() && (status == "Active" || status == "All") && (tag_type == "Annotated" || tag_type == "All")
            {
                for tag in json_annotated.keys()
                {tags.insert(tag.to_string(),  ObjRef {current: json_annotated[tag]["current"].clone(), previous: json_annotated[tag]["previous"].clone()});}
            }

            if !json_retired.is_empty() && (status == "Retired" || status == "All") 
            {
                for tag in json_retired.keys()
                {tags.insert(tag.to_string(),  ObjRef {current: json!(null), previous: json_retired[tag].clone()});}
            }

            let mut keys: Vec<&String> = tags.keys().collect();
            keys.sort();

            for tag in keys
            {
                if data != "" {data += "\n";}

                if format == "Event"
                {
                    if tags[tag].previous.is_null()     {data += "C\t";}
                    else if tags[tag].current.is_null() {data += "R\t";}
                    else                                {data += "M\t";}
                }

                if format == "Long"
                {
                    if json_annotated.contains_key(tag)  {data += "A\t";}
                    else if json_light.contains_key(tag) {data += "L\t";}
                    else                                 {data += " \t";}                                        
                }

                data += &tag;

                if format == "Index"
                {
                    if !tags[tag].current.is_null()
                    {data += &("\t".to_owned() + &tags[tag].current.as_str().unwrap());}
                }
                else if format == "Long"
                {
                    data += "\t";

                    if !tags[tag].previous.is_null()
                    {data += &tags[tag].previous.as_str().unwrap();}
                    data += ":";
                    if !tags[tag].current.is_null()
                    {data +=  &tags[tag].current.as_str().unwrap();}
                }
            }
        }

        if let Some(out) = inspect.get_one::<String>("outfile")
        {
            let out_file = define_file_name(input, out, &extention);
            fs::write(out_file, data.clone()).expect("inspect, run: Unable to write file");
        }
        else {print!("{}", data);}
    }
}

/* split_import
 *============================================
 * Purpose:     split the import into pack and json
 * Input:       import file
 * Results:     json and pack file
 * Notes:       
 */
pub fn split_import(input: &str) -> (Vec<u8>, Vec<u8>)
{
    let sync_pack: &Path = Path::new(input);
    
    let mut obj  = File::open(sync_pack).unwrap();
    let mut data = vec![];
    
    let _ = obj.read_to_end(&mut data);

    let mut json_bit: Vec<u8> = vec![];
    let mut pack_bit: Vec<u8> = vec![];

    let mut found: bool  = false;
    for bit in data
    {
        if bit == 10    {found = true;}
        else if !found  {json_bit.push(bit);}
        else            {pack_bit.push(bit);}
    } 

    return (json_bit, pack_bit)
}


/* write_pack
 *============================================
 * Purpose:     write a pack file either to file or stdout
 * Input:       pack contents, out file or None
 * Results:     NONE
 * Notes:       
 */
fn write_pack(pack: &Vec<u8>, out_path: &Option<PathBuf>)
{
    let encoding: String = String::from_utf8(pack.to_vec()).unwrap();
    let binary: Vec<u8>  = BASE64_STANDARD.decode(encoding).unwrap();
    if let Some(path) = out_path
    {
        let mut pack_file = File::create(path).unwrap();
        pack_file.write(&binary).unwrap();
    }
    else {io::stdout().write_all(&binary).unwrap();}
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

        file_name = file_name.substring(0, file_name.rfind(".").unwrap()).to_string();

        path.push(file_name + "." + extension);
    }

    return path
}
