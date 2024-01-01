/* Git-Pack-Sync::import
 *============================================
 * Purpose:     Perform an import of pack files
 */

use clap::ArgMatches;

use std::io::Write;
use std::error::Error;
use std::process::Command as cmd;
use std::process::Stdio;
use std::collections::HashMap;

use grep_searcher::sinks::UTF8;
use grep_searcher::SearcherBuilder;
use grep_regex::RegexMatcher;

use serde_json::Value;

use base64::prelude::BASE64_STANDARD;
use base64::Engine;

use substring::Substring;

use crate::inspect::split_import;
use git;


/* run
 *============================================
 * Purpose:     run an import
 * Input:       ArgMAtches
 * Results:     NONE
 * Notes:       
 */
pub fn run(sub: &ArgMatches) 
{
    let mut remote_path: &str = "";

    if sub.get_one::<String>("remote_path") != None
    {remote_path = sub.get_one::<String>("remote_path").unwrap();}
    


    let import:    &str = sub.get_one::<String>("import").unwrap();
    let importing: &str = sub.get_one::<String>("objects").unwrap();
    let removing:  &str = sub.get_one::<String>("retired").unwrap();
    let overwrite: &str = sub.get_one::<String>("overwrite").unwrap(); 
    let include: Option<String> = sub.get_one::<String>("include").cloned();
    let exclude: Option<String> = sub.get_one::<String>("exclude").cloned();



    let json: Value = serde_json::from_str(&import_object(&import, remote_path)).unwrap();

    let mut missing_commit: Vec<String> = vec![];
    for commit in json["requiredCommits"].as_array().unwrap()
    { // determine if all commits that are required is available
        let id: String  = commit.as_str().unwrap().to_string();
        
        if !git::sha_type(&id, remote_path).is_some()
        {missing_commit.push(id.clone());}
    }


    if !missing_commit.is_empty()
    {panic!("import, run: Missing Commit:\n   {}", &missing_commit.join("   \n"));}


    let local: (HashMap<String, String>, HashMap<String, String>, HashMap<String, String>) = get_local(remote_path);

    let branches = json["branch"].as_object().unwrap();
     
    if importing == "Both" || importing == "Branches"
    {  
        let active_branch = branches["active"].as_object().unwrap();
        if !active_branch.is_empty()
        {
            let mut branches_vec: Vec<u8> = vec![]; // building a search array
            for branch in active_branch.keys()
            {
                for bit in branch.as_bytes()
                {branches_vec.push(*bit);}
                branches_vec.push(b'\n');
            }
            
            if exclude != None
            {
                for value in exclude.unwrap().split(",")
                {
                    if value != ""
                    {
                        let mut sink: Vec<u8> = vec![];

                        let _ = grep(&branches_vec, value, true, &mut sink); //Those that match should be removed
                        
                        branches_vec = vec![];
                        for byte in sink  {branches_vec.push(byte);}
                    }
                }
            }

            if branches_vec.is_empty() {return;} 
            if include != None         
            {
                let test_branch: Vec<u8> = branches_vec.clone();
                for value in include.unwrap().split(",")
                {
                    if value != ""
                    {
                        let mut sink: Vec<u8> = vec![];

                        let _ = grep(&test_branch, value, false, &mut sink); //Those that do not match should be removed
                        branches_vec = vec![];
                        for byte in sink  {branches_vec.push(byte);}
                    }
                }
            }

            if branches_vec.is_empty() {return;}

            let mut branch_names: Vec<String> = vec![];
            for  name in String::from_utf8(branches_vec.clone()).unwrap().split("\n")
            {
                if !branch_names.contains(&name.to_string()) && name != ""
                {branch_names.push(name.to_string())}
            }



            for branch_name in branch_names
            {
                let name: String = branch_name.to_string();
                println!("{}", name);
                let json_branch = active_branch[&name].as_object().unwrap();
                if local.0.contains_key(&name) 
                {   
                    let local_ref: &str = local.0.get(&name).unwrap();

                    // if the local ref != current and
                    // either local == previous or no previous or hard replacment or Mixed replacement with local being an ancestor 
 
                    if json_branch["current"].as_str().unwrap() != local_ref &&  
                       (json_branch["previous"].as_str().unwrap() == local_ref || json_branch["previous"].is_null() || overwrite == "Hard" || 
                       (overwrite == "Mixed" && git::is_ancestor(&json_branch["current"].as_str().unwrap(), &local_ref, remote_path)))
                    {
                        if !git::update_ref(json_branch["current"].as_str().unwrap(), &("refs/heads/".to_owned() + &name), remote_path)
                        {panic!("import, run: git push --all, error");}
                    }
                    else if json_branch["current"].as_str().unwrap() != local_ref
                    {
                        if !git::is_ancestor(&json_branch["current"].as_str().unwrap(), local_ref, remote_path)
                        {eprintln!("import, run: Warning, {} commit is not the expected version but it is a fast forward commit. Will not update unless mixed overwrite", name);}
                        else
                        {eprintln!("import, run: Warning, {} commit is not a fast forward or expected version. Will not update unless hard overwrite", name);}
                    }
                }
                else 
                {
                    let output = shared::cmd("git", ["branch", &name, &json_branch["current"].as_str().unwrap()].to_vec(), remote_path);
                    if !output.status
                    {panic!("import, run: failed to create branch, {}", name);}
                }
            }
        }
    }
    
    if removing == "Both" || removing == "Branches" 
    {
        let branches_retired = branches["retired"].as_object().unwrap();
        if !branches_retired.is_empty()
        {
            for branch in branches["retired"].as_object().unwrap().keys()
            {
                if local.0.contains_key(branch)
                {
                    let local_ref:  &str = local.0.get(branch).expect("REASON");
                    let branch_ref: &str = &branches_retired[branch].as_str().unwrap();

                    // local ref == last exported or hard or mixed and commit is an ancestor
                    if branch_ref == local_ref ||overwrite == "Hard" ||  
                        (overwrite == "Mixed" && git::is_ancestor(branch_ref, local_ref, remote_path))
                    {
                        if !git::delete_branch (branch, remote_path)
                        {panic!("import, run: failed to delete branch, {}", branch);}
                    }
                    else if overwrite == "Mixed"
                    {eprintln!("import, run: Warning, {}'s commit is not a fast forward or expected version. Will not removed unless hard forced", branch);}
                    else 
                    {eprintln!("import, run: Warning, {}'s commit is not expected version. Will not removed unless hard forced", branch);}
                }
            }
        }
    }
    
    let tags = json["tag"].as_object().unwrap();
    
    if importing == "Both" || importing == "Tags"
    {
        let light_tag = tags["light"].as_object().unwrap();
        if !light_tag.is_empty()
        {
            for tag in light_tag.keys()
            {
                let obj = light_tag[tag].as_object().unwrap();
                if local.1.contains_key(tag) //light tags
                { 
                    let tag_ref: &str = local.1.get(tag).unwrap();
                    if tag_ref == obj["current"].as_str().unwrap() { continue;}
                    else if overwrite == "Hard" || obj["previous"].is_null() || obj["previous"].as_str().unwrap() == tag_ref 
                    {
                        if !git::delete_tag (tag, remote_path)
                        {panic!("import, run: failed to delete tag {}", tag);}
                    }
                    else 
                    {
                        eprintln!("import, run: Warning, {} exists and is not being updated. Will not update unless hard overwrite", tag);
                        continue;     
                    }           
                }
                else  if local.2.contains_key(tag) //annotated tags
                {
                    if overwrite == "Hard" || overwrite == "Mixed"
                    {
                        if !git::delete_tag (tag, remote_path)
                        {panic!("Push-Extended remote: git push --all, error");}
                    }
                    else
                    {
                        eprintln!("import, run: Warning, {} exists as an annotated tag. Will not update unless mixed or hard overwrite", tag);
                        continue;     
                    } 
                }
                
                if !git::create_light_tag (tag, &obj["current"].as_str().unwrap(), remote_path) 
                {panic!("import, run: failed to create light tag, {}", tag);}
            }
        }
    }
    
    if importing == "Both" || importing == "Tags" 
    {
        let annotated_tags = tags["annotated"].as_object().unwrap();
        if !annotated_tags.is_empty()
        { 
            for tag in annotated_tags.keys()
            {
                let obj = annotated_tags[tag].as_object().unwrap();
                if local.2.contains_key(tag) //annotated tags
                {
                    let tag_ref: &str = local.1.get(tag).expect("REASON");
                    if tag_ref == obj["current"].as_str().unwrap() { continue;}
                    else if overwrite == "Hard" || obj["previous"].is_null() || obj["previous"].as_str().unwrap() == tag_ref
                    {
                       if !git::delete_tag (tag, remote_path)
                        {panic!("import, import: failed to delete tag {}", tag);}
                    }
                    else 
                    {
                        eprintln!("import, run: Warning, {} exists and is not being updated. Will not update unless hard overwrite", tag);
                        continue;     
                    }           
                }
                else if local.1.contains_key(tag) //light tags
                {
                    if overwrite == "Hard" || overwrite == "Mixed"
                    {
                        if !git::delete_tag (tag, remote_path)
                        {panic!("Push-Extended remote: git push --all, error");}
                    }
                    else 
                    {
                        eprintln!("import, run: Warning, {} exists as an light tag. Will not update unless mixed or hard overwrite", tag);
                        continue;     
                    } 
                }
                
                if !git::update_ref(obj["current"].as_str().unwrap(), &("refs/tags/".to_owned() + tag), remote_path)
                {panic!("import, run: failed to create annotated tag {}", tag);}
            }
        }
    }
    
    if removing == "Both" || removing == "Tags"
    {
        let retired_tags = tags["retired"].as_object().unwrap();
        if !retired_tags.is_empty()
        {
            for tag in retired_tags.keys()
            {
                let retired_ref: &str = retired_tags[tag].as_str().unwrap();
                
                if (local.1.contains_key(tag) && ( local.1.get(tag).unwrap() == retired_ref || overwrite != "Soft")) || 
                   (local.2.contains_key(tag) && ( local.2.get(tag).unwrap() == retired_ref || overwrite != "Soft"))
                { 
                    if !git::delete_tag (tag, remote_path)
                    {panic!("import, run: failed to delete {}", tag);}
                }
                else if local.1.contains_key(tag) || local.2.contains_key(tag)
                {eprintln!("import, run: Warning, {} exists but ref is not expected. Will not removed unless mixed or hard overwrite", tag);}
            }
        }
    }
}

/* import_object
 *============================================
 * Purpose:     unpack the low side objects into high
 * Input:       inport and remote path
 * Results:     json data
 * Notes:       
 */
fn import_object(input: &str, remote_path: &str) -> String
{
    let pack_data: (Vec<u8>, Vec<u8>) = split_import(input); 

    if pack_data.1.len()>4
    {
        let mut cmd = cmd::new("git");
        cmd.arg("unpack-objects").stdin(Stdio::piped());

        if remote_path != "" {cmd.current_dir(remote_path);}

        let mut pack_cmd = cmd.spawn().unwrap();

        let stdin = pack_cmd.stdin.take();
        let _ = stdin.unwrap().write_all(&BASE64_STANDARD.decode(&pack_data.1).unwrap());
  

        let result = pack_cmd.wait().unwrap();

        if !result.success()
        {panic!("import, import_object: failed to import the pack file");}
    }

    return String::from_utf8(pack_data.0).unwrap()
}


/* get_local
 *============================================
 * Purpose:     Get local branches, light and annotated tags
 * Input:       path
 * Results:     List of branches, light and annotated tags
 * Notes:       
 */
fn get_local(path: &str) -> (HashMap<String, String>, HashMap<String, String>, HashMap<String, String>)
{
    let mut branches:HashMap<String, String>       = HashMap::new();
    let mut light_tags:HashMap<String, String>     = HashMap::new();
    let mut annotated_tags:HashMap<String, String> = HashMap::new();

    if let Some(refs) = git::all_ref(path)
    {
        for value in refs
        {
            let id: String  = value.substring(0, value.find(" ").unwrap()).to_string();
            let obj: String = value.substring(value.find(" ").unwrap()+1, value.len()).to_string();

            if obj.starts_with("refs/heads/") 
            {branches.insert(obj.substring(11, obj.len()).to_string(),  id.clone());}

            if obj.starts_with("refs/tags/") 
            {

                if let Some(sha_type) = git::sha_type(&id, path)
                {
                    if sha_type == "commit"
                    {light_tags.insert(obj.substring(10, obj.len()).to_string(), id);}
                    else 
                    {annotated_tags.insert(obj.substring(10, obj.len()).to_string(), id);}
                }
                else {panic!("import, get_local: failed to get sha_type for {}", id);}
            }

        }    
        return (branches, light_tags, annotated_tags)
    }

    panic!("import, get_local: Failed to get types");
}



/* grep
 *============================================
 * Purpose:     Perform a Linux mimic grep
 * Input:       Content, Matching, is this an excluded, final sets
 * Results:     Results and errors
 * Notes:       This is based on another code, striped down. May not need to return error
 */
fn grep(content: &[u8], matching: &str, exclude: bool, sink: &mut Vec<u8>) -> Result<(), Box<dyn Error>>
{
    let matcher     = RegexMatcher::new(matching)?;

    let mut searcher = SearcherBuilder::new();
    searcher.invert_match(exclude);
    searcher.build().search_slice(&matcher, content, UTF8(|_lnum, line| {

        for i in line.as_bytes() {sink.push(i.clone());}
        sink.push(10);
        
        Ok(true)
    }))?;

    return Ok(())
}