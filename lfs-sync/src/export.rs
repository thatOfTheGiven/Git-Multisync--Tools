/* Git-lfs-Sync::export
 *============================================
 * Purpose:     Perform an export of pack files
 */



use clap::ArgMatches;

use std::env;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use std::collections::HashMap;

use substring::Substring;

use regex::Regex;

use chrono::{DateTime, Utc};

use serde_json::{json, Map, Value};

use base64::Engine;
use base64::prelude::BASE64_STANDARD;

use crate::snapshot::gen;

use shared::cmd;
use shared::file_path;
use git;
use lfs;



/* run
 *============================================
 * Purpose:     run an export
 * Input:       ArgMAtches, timestamp, datetime
 * Results:     NONE
 * Notes:       
 */
pub fn run(sub: &ArgMatches, time_stamp: &str, now: DateTime<Utc>)
{
    let mut remote_path: &str = "";
    let out_path: &str;
    let mut contents:HashMap<String, String> = HashMap::new(); 

    let mut build_json = Map::new(); 
    let export: ArgMatches;

    if sub.get_one::<String>("remote_path") != None
    {remote_path = sub.get_one::<String>("remote_path").unwrap();}

    build_json.insert("export".to_string(), json!(time_stamp));
    build_json.insert("mapping".to_string(), json!({}));


    if sub.subcommand_matches("All").is_some()
    {
        export = sub.subcommand_matches("All").unwrap().clone();
        build_json.insert("type".to_string(), json!("All"));        
    }
    else if sub.subcommand_matches("Snapshot").is_some()
    {
        export = sub.subcommand_matches("Snapshot").unwrap().clone();
        build_json.insert("type".to_string(), json!("Snapshot"));
    }
    else if sub.subcommand_matches("Files").is_some()
    {
        export = sub.subcommand_matches("Files").unwrap().clone();
        build_json.insert("type".to_string(), json!("Files"));        
    }
    else if sub.subcommand_matches("Timestamp").is_some()
    {
        export = sub.subcommand_matches("Timestamp").unwrap().clone();
        build_json.insert("type".to_string(), json!("Timestamp"));
    }
    else if sub.subcommand_matches("Branch").is_some()
    {
        export = sub.subcommand_matches("Branch").unwrap().clone();
        build_json.insert("type".to_string(), json!("Branch"));

    }
    else if sub.subcommand_matches("Tag").is_some()
    {
        export = sub.subcommand_matches("Tag").unwrap().clone();
        build_json.insert("type".to_string(), json!("Tag"));
    }    
    else if sub.subcommand_matches("Diff").is_some()
    {
        export = sub.subcommand_matches("Diff").unwrap().clone();
        build_json.insert("type".to_string(), json!("Diff"));
    }
    else 
    {panic!("export run: unexpected state");}


    out_path = export.get_one::<String>("OutPath").unwrap();

    if sub.subcommand_matches("Snapshot").is_some() ||  sub.subcommand_matches("All").is_some()
    {
        let current_refs : PathBuf = file_path("lfs", &time_stamp, "snp", out_path );
        gen(remote_path, Some(&current_refs));
        
        for (_i, value) in fs::read_to_string(current_refs).unwrap().split('\n').enumerate() 
        {
            let id:  String = value.substring(0, value.find("-").unwrap()-1).to_string();
            let obj: String = value.substring(value.find("-").unwrap()+2, value.len()).to_string();

             
            contents.insert(id, obj);
        }


        if sub.subcommand_matches("Snapshot").is_some()
        {
            for (_i, value) in fs::read_to_string(export.get_one::<String>("Input").unwrap().to_string()).unwrap().split('\n').enumerate() 
            {
                let id:  String = value.substring(0, value.find("-").unwrap()-2).to_string();
                
                contents.remove(&id);
            }
        }
    }
    else 
    {
        let mut path_mapping: HashMap<String, Vec<String>> = HashMap::new();
        let mut commits_tree: HashMap<String, Vec<String>> = HashMap::new();


        for line in lfs::list(remote_path).unwrap()
        {
            let id:  String = line.substring(0, line.find("-").unwrap()+1).to_string();
            let obj: String = line.substring(line.find(" ").unwrap()+2, line.len()).to_string();

            if !path_mapping.contains_key(&obj)
            {path_mapping.insert(obj.clone(), vec![]);}

            path_mapping.get_mut(&obj).unwrap().push(id);
        }


        if sub.subcommand_matches("Files").is_some()
        {
            for file in export.get_many::<String>("Files").unwrap()
            {
                if path_mapping.contains_key(file)
                {
                    for sha in path_mapping[file].clone()
                    {
                        contents.insert(sha, file.to_string());
                    }
                }
                else 
                {panic!("git-lfs-sync, export: failed to find file in lfs.");}
            }
        }
        else
        { 
            let mut commits: Vec<String> = vec![];
            if sub.subcommand_matches("Timestamp").is_some()
            {
                let date: String;
                let time: String;

                if let Some(value) = export.get_one::<String>("Date")
                {
                    let re = Regex::new(r"^(19|20|21)[0-9]{2}-(1[0-2]|0[1-9])-(3[01]|[12][0-9]|0[1-9])$").unwrap(); //being optimistic 100 years or uses

                    if re.is_match(value) {date = value.to_string()}
                    else 
                    {panic!("export, run: {:?} is not a valid date format (YYYY-MM-DD)", value);}
                }
                else  {date  = now.format("%Y-%m-%d").to_string();}

                if let Some(value) = export.get_one::<String>("Time")
                {
                    let re = Regex::new(r"^(2[0-3]|[0-1][0-9])(:[0-5][0-9]){2}$").unwrap();


                    if re.is_match(value) {time = value.to_string()}
                    else 
                    {panic!("export, run: {:?} is not a valid date format (HH:mm:ss)", value);}
                }
                else  {time = "00:00:00".to_string()}

                println!("-after=\"{}T{}\"", date, time);

                let content = shared::cmd("git", ["log", "--all", &("--after=\"".to_owned() + &date + "T" + &time + "+00:00\""), "--pretty=format:%H=%P"].to_vec(), remote_path); 
                for line in content.out.split("\n")
                {
                    let commit      = line.substring(0, line.find("=").unwrap());
                    let mut parents = line.substring(line.find("=").unwrap()+1, line.len());
                    parents         = parents.substring(0, parents.find("=").unwrap());

                    let mut parent_list: Vec<String> = vec![]; 
                    for parent in parents.split(" ")
                    {parent_list.push(parent.to_string());}

                    commits_tree.insert(commit.to_string(), parent_list);
                    commits.push(commit.to_string());
                }
            }    
            else if sub.subcommand_matches("Branch").is_some()
            {        
                let mut rooted_commits: Vec<String> = vec![];

                if let Some(cli_roots) = export.get_many::<String>("root")
                {  
                    for (_, root) in cli_roots.map(|s| s.to_string()).enumerate()
                    {
                        let root_ref = get_ref(&root, "NONE", remote_path);

                        if root_ref.1 == "tag"
                        {rooted_commits.push(tag_commit(&root_ref.0, remote_path).0);}
                        else {rooted_commits.push(root_ref.0);}
                    }
                }

                let visted: Vec<String> = vec![];

                for branch in export.get_many::<String>("Name").unwrap()
                {
                    let commit_id: &str = &get_ref(branch, "branch", remote_path).0;
                    commits.push(commit_id.to_string()); 

                    let tree = find_commits (commit_id.to_string(), &rooted_commits, &visted, remote_path);
                    for id in tree.keys()
                    {commits_tree.insert(id.to_string(), tree[id].clone());}

                }
            }
            else  
            {
                let mut rooted_commit: Vec<String> = vec![];
                if let Some(cli_roots) = export.get_many::<String>("root")
                {              
                    for (_, root) in cli_roots.map(|s| s.to_string()).enumerate()
                    {
                        let root_ref = get_ref(&root, "NONE", remote_path);

                        if root_ref.1 == "tag"
                        {rooted_commit.push(tag_commit(&root_ref.0, remote_path).0);}
                        else {rooted_commit.push(root_ref.0);}
                    }            
                }

                for obj in export.get_many::<String>("Selection").unwrap()
                {
                    let commit_id: String;
                    let obj_ref = get_ref(obj, "NONE", remote_path);

                    if obj_ref.1 == "tag"  {commit_id = tag_commit(&obj_ref.0, remote_path).0;}
                    else {commit_id = obj_ref.0;}
                
                    commits.push(commit_id.to_string());


                    if rooted_commit.is_empty()
                    {
                        if let Some(parents) = git::get_parents(&commit_id, remote_path)
                        { commits_tree.insert(commit_id.to_string(), parents);}
                    }                    
                    else
                    {
                        let tree = find_commits (commit_id.to_string(), &rooted_commit, &[].to_vec(), remote_path);

                        for id in tree.keys()
                        {commits_tree.insert(id.to_string(), tree[id].clone());}
                    }
                }
            }


            for commit in commits
            {
                for file in affected(&commit, &mut commits_tree.clone(),  remote_path)
                {
                    if path_mapping.contains_key(&file)
                    {
                        for sha in path_mapping[&file].clone()
                        {
                            contents.insert(sha, file.clone());
                        }
                    }
                }
            }
        }
    }

    build_json.insert("mapping".to_string(), json!({}));
    if !contents.is_empty()
    {
        let shas: Vec<String> = contents.clone().into_keys().collect();
        for sha in &shas
        {
            build_json["mapping"].as_object_mut().unwrap().insert(sha.clone(), json!(contents[sha]));
        }

        let json = Value::Object(build_json);
        let mut out: File = File::create(file_path("lfs", time_stamp, "exp", out_path)).unwrap();
            
        let _ = out.write(&json.to_string().into_bytes());

        
        encode_objects (&shas, &out,  remote_path);
    }
    else {println!("No lfs objects detected. Not generating export.")}      
}



  


/* find_commits
 *============================================
 * Purpose:     find commits tree
 * Input:       commit, rooted_commits, visited, path
 * Results:     commit tree
 * Notes:       
 */
fn find_commits (commit: String, rooted_commits: & Vec<String>, visited:  & Vec<String>, path: & str)  -> HashMap<String, Vec<String>>
{
    let mut process_list: Vec<String>  = [commit.clone()].to_vec();
    let mut commits: HashMap<String, Vec<String>> = HashMap::new(); 
  
    while !process_list.is_empty()
    { 
        let current_commit: &String = &process_list.pop().unwrap();

        if visited.contains(current_commit) || commits.contains_key(current_commit)
        {continue;}

        let mut transfered: bool = false;
        if is_rooted(&current_commit, rooted_commits, path)
        {transfered = true}

        if !transfered
        {  
            if let Some(mut parents) = git::get_parents(&commit, path)
            {
                process_list.append(&mut parents);
                commits.insert(current_commit.to_string(), parents);
            }
            else
            {panic!("export, find_commits: commit id, {}", current_commit);}            
        }
    }

    return commits;
}


/* scan_changes
 *============================================
 * Purpose:     find change from a commit path
 * Input:       commit, commit tree, required commits, seen commits, path
 * Results:     list objects to export
 * Notes:       
 */
fn affected(commit: &str, commits: &mut HashMap <String, Vec<String>>,  path: & str) -> Vec<String> 
{    
    let mut changes: Vec<String> = vec![];
    let mut parents_queue: Vec<String> = [commit.to_string()].to_vec();
    while !parents_queue.is_empty()
    {
        let parent_commit: String = parents_queue.pop().unwrap(); 

        let mut commits_queue: Vec<String> = [parent_commit.to_string()].to_vec();
        while !commits_queue.is_empty()
        { 
            let current_commit: String = commits_queue.pop().unwrap();

            if commits.contains_key(&current_commit)
            {
                let parents: &Vec<String> = &commits[&current_commit];               
                
                if parents.len() == 0
                {
                    if commit != current_commit
                    {changes.append(&mut get_objs (commit, &current_commit, path));}
                    else {changes.append(&mut get_objs (commit, "", path));}
                }
                else if parents.len() == 1
                {commits_queue.push(parents[0].to_string());}
                else 
                {
                    changes.append(&mut get_objs(&parent_commit, &parents[0], path));

                    for  parent in parents
                    {parents_queue.push(parent.to_string());}
                }

                commits.remove(&current_commit);
            }
            else if current_commit != parent_commit 
            {
                changes.append(&mut get_objs(&parent_commit, &current_commit, path)); 

            }

        }
    }

    changes.dedup();
    return changes;
}



/* get_ref
 *============================================
 * Purpose:     given input, determine type and commit id
 * Input:       ePrfix, timestamp, extension, root
 * Results:     commit, type
 * Notes:       
 */
fn get_ref(obj: &str, expected_type: &str, path: &str) -> (String, String)
{    
    let branches = git::list_branches(path).unwrap();
    let tags = git::list_tags(path).unwrap();

    let commit_rx = Regex::new(r"^[0-9a-f]{4,40}([0-9a-f]{1,24}([0-9a-f]{1,88})?)?$").unwrap();    

    if !branches.contains(&obj.to_string()) && !tags.contains(&obj.to_string()) && commit_rx.is_match(obj)
    {
        if let Some(rev) = git::get_commit(obj, path)
        {return (rev, "commit".to_string());}
                
        panic!("export, get_ref: failed to find tag/branch/commit id {}", obj.to_string());
    }

    let obj_name: String;
    let obj_type: String;
    if !obj.starts_with("refs/tags") && expected_type != "tag" && 
        (branches.contains(&obj.to_string()) || obj.starts_with("refs/heads"))
    {
        if obj.starts_with("refs/heads")
        {obj_name = obj.substring(11, obj.len()).to_string();}  //take off refs/heads
        else {obj_name = obj.to_string();}

        if ! branches.contains(&obj_name)
        {panic!("export, get_ref: failed to find branch {}", obj_name);}
        obj_type = "branch".to_string();
    }
    else if !obj.starts_with("refs/branches") && expected_type != "branch" && 
        (tags.contains(&obj.to_string()) || obj.starts_with("refs/tags"))
    {
      
        if obj.starts_with("refs/tags")
        {obj_name = obj.substring(10, obj.len()).to_string();} //take off refs/tags
        else {obj_name = obj.to_string();}

        if ! tags.contains(&obj_name)
        {panic!("export, get_ref: failed to find tag {}", obj_name);}

        obj_type = "tag".to_string();
    } 
    else
    {panic!("export, get_ref: failed to find tag/branch/commit id {}", obj.to_string());}

    if let Some(ref_sha) = git::show_ref(&obj_name, path)
    {return (ref_sha, obj_type);}

    panic!("export, get_ref: failed to find the ref");
}

/* tag_commit
 *============================================
 * Purpose:     determine tag type and commit id
 * Input:       ref_id, path to git repo
 * Results:     commit id, false if light weight, true if annotated
 * Notes:       
 */
fn tag_commit(ref_id: &str, path: &str) -> (String, bool)
{
    let cat_file = git::sha_type(ref_id, path).unwrap();

    if cat_file == "commit"
    {return(ref_id.to_string(), false);}
            
    let rev_list = git::get_commit(ref_id, path).unwrap();
    return(rev_list, true);
}


/* is_rooted
 *============================================
 * Purpose:     determine if commit is a rooted commit
 * Input:       commit, list of rooted commits, path
 * Results:     true is rooted, false if not
 * Notes:       
 */
fn is_rooted(commit: &str, rooted_commits: & Vec<String>, path: &str) -> bool
{
    for rooted_commit in rooted_commits
    {
        if &commit == rooted_commit || git::is_ancestor(rooted_commit, commit, path)
        {return true}
    }

    return false
}


/* get_objs
 *============================================
 * Purpose:     get objects that changes in a commit
 * Input:       commit, ancestor, path
 * Results:     vector of objects names 
 * Notes:       
 */
pub fn get_objs(commit: &str, ancestor: &str, path: &str) -> Vec<String>
{
    let arg: String; 
    if ancestor != "" {arg = ancestor.to_owned() + ".." + commit;}
    else {arg = commit.to_string()}

    let output = cmd("git", ["rev-list", "--objects", "--filter=object:type=blob", &arg ].to_vec(), path);
    if output.status
    {
        let mut objs: Vec<String> = vec![];
        for obj in output.out.split("\n")
        {
            if let Some(index) = obj.find(" ")
            {objs.push(obj.substring(index+1, obj.len()).to_string());}
        }


        return objs
    }

    panic!("export, pars_content: failed to get objects ");
}


/* encode_objects
 *============================================
 * Purpose:     produce a pack of objects need to be export
 * Input:       list of objects, remote path
 * Results:     encoded pack data
 * Notes:       
 */
fn encode_objects (objects: &Vec<String>, mut out: &File,  remote_path: &str)
{   
    let mut objects_root: PathBuf;

    if remote_path != ""
    {objects_root = PathBuf::new().join(".git");}
    else {objects_root = Path::new(".git").to_path_buf();}


    if !objects_root.exists() 
    {objects_root = env::current_dir().unwrap()}

    objects_root = objects_root.join("lfs").join("objects");

    if !objects_root.exists() 
    {panic!("Failed to find root dir, {:?}", objects_root)}

    for obj in objects
    {println!(":|{}", obj);
        let _ = out.write(&[b'\n']);
        let mut obj_path: PathBuf  = objects_root.join(obj.substring(0, 2));
        if !obj_path.exists()
        {panic!("Failed to find {:?}", obj_path)}

        obj_path = obj_path.join(obj.substring(2, 4));
        if !obj_path.exists()
        {panic!("Failed to find {:?}", obj_path)}

        obj_path = obj_path.join(obj);
        if !obj_path.exists()
        {panic!("Failed to find {:?}", obj_path)}

        println!("{:?}", obj_path);
        
        let mut obj_file  = File::open(obj_path).unwrap();
        let mut data = vec![];
    
        let _ = obj_file.read_to_end(&mut data);

        let _ = out.write(obj.as_bytes());
        let _ = out.write(b":");
        let _ = out.write(BASE64_STANDARD.encode(data).as_bytes());
    }
}