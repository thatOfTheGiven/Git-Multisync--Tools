/* Git-Pack-Sync::export
 *============================================
 * Purpose:     Perform an export of pack files
 */


use clap::ArgMatches;

use std::collections::HashMap;
use std::fs;
use std::io::{Write};
use std::path::{PathBuf};
use std::process::Stdio;
use std::process::Command as cmd;

use substring::Substring;

use regex::Regex;

use chrono::{DateTime, Utc};

use serde_json::{json, Map, Value};

use base64::Engine;
use base64::prelude::BASE64_STANDARD;

use crate::snapshot::gen;
use git;

use shared::file_path;

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

    if sub.get_one::<String>("remote_path") != None
    {remote_path = sub.get_one::<String>("remote_path").unwrap();}


    let mut export_objects: Vec<String> = vec![];
    let mut build_json                  = Map::new();    

    let mut commits_tree: HashMap <String, Vec<String>> = HashMap::new();
    let mut commits: Vec<String>                        = vec![];
    let mut required_commits: Vec<String>               = vec![];
    let mut visted: Vec<String>                         = vec![]; 

    let export: ArgMatches;

    build_json.insert("export".to_string(), json!(time_stamp));
    build_json.insert("tag".to_string(), json!({"light": {}, "annotated": {}, "retired": {}}));
    build_json.insert("branch".to_string(), json!({"active": {}, "retired": {}}));

                  
    if sub.subcommand_matches("Snapshot").is_some()
    {
        export = sub.subcommand_matches("Snapshot").unwrap().clone();
        build_json.insert("type".to_string(), json!("Snapshot"));
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

    let mut tags: HashMap <String, (String, String)>     = HashMap::new();
    let mut retired_tags: HashMap <String, String>       = HashMap::new();
    let mut branches: HashMap <String, (String, String)> = HashMap::new();
    let mut retired_branches: HashMap <String, String>   = HashMap::new();



    
    if sub.subcommand_matches("Snapshot").is_some()
    {            
        let current_refs : PathBuf = file_path("pack", &time_stamp, "snp", out_path );
        //let out_path: Option<&File> = Some(&current_refs);

        let mut primary_branch: Vec<String> = vec![];

        if let Some(cli_primary) = export.get_many::<String>("primary")
        { 
            for (_, primary) in cli_primary.map(|s| s.to_string()).enumerate()
            {primary_branch.push(get_ref(&primary, "BRANCH", remote_path).0);}
        }
        else 
        {
            let default_branch = git::default_branch(remote_path).unwrap(); 
            primary_branch.push(get_ref(&default_branch, "BRANCH", remote_path).0);
        }

        
        gen(remote_path, Some(&current_refs));

        let mut snapshot_mapping: HashMap<String, String> = HashMap::new();
        for (_i, value) in fs::read_to_string(export.get_one::<String>("Input").unwrap().to_string()).unwrap().split('\n').enumerate() 
        {
            let id:  String = value.substring(0, value.find(" ").unwrap()).to_string();
            let obj: String = value.substring(value.find(" ").unwrap()+1, value.len()).to_string();

            if obj.starts_with("refs/heads/") 
            {visted.push(id.clone())}

            snapshot_mapping.insert(obj, id);
        }

        let contents =  fs::read_to_string(current_refs).unwrap();
        for value in contents.split('\n')
        {
            if value == ""
            {continue;}
            
            let id:  String = value.substring(0, value.find(" ").unwrap()).to_string();
            let obj: String = value.substring(value.find(" ").unwrap()+1, value.len()).to_string();

            let mut previous_id: String = "".to_string();
            if snapshot_mapping.contains_key(&obj)
            {
                previous_id = snapshot_mapping.get(&obj).unwrap().to_string();
                snapshot_mapping.remove(&obj);
            }
        
            
            if previous_id != id
            {
                if obj.starts_with("refs/heads/") 
                {
                    branches.insert(obj.substring(11, obj.len()).to_string(),  (previous_id.to_string(), id.clone()));

                    if !commits.contains(&id)
                    {commits.push(id.clone())}

                    let tree: HashMap<String, Vec<String>>;

                    if primary_branch.contains(&id)
                    {tree = find_commits(id.to_string(),  &[].to_vec(), &visted, remote_path);}
                    else
                    {tree = find_commits (id.to_string(), &primary_branch, &visted, remote_path);}

                    for id in tree.keys()
                    {
                        if !commits_tree.contains_key(id)
                        {commits_tree.insert(id.to_string(), tree[id].clone());}
                    }
                }

                if obj.starts_with("refs/tags/") 
                {tags.insert(obj.substring(10, obj.len()).to_string(),  (previous_id.to_string(), id));}
            }
        }

        for obj in snapshot_mapping.clone().into_keys()
        {
            if obj.starts_with("refs/tags/") 
            {retired_tags.insert(obj.substring(10, obj.len()).to_string(), snapshot_mapping.get(&obj).expect("REASON").to_string());}

            if obj.starts_with("refs/heads/") 
            {retired_branches.insert(obj.substring(11, obj.len()).to_string(), snapshot_mapping.get(&obj).expect("REASON").to_string());}
        }
    }
    else if sub.subcommand_matches("Timestamp").is_some()
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

        let content = shared::cmd("git", ["log", "--all", &("--after=\"".to_owned() + &date + "T" + &time + "+00:00\""), "--pretty=format:%H=%P=%D"].to_vec(), remote_path); 
        for line in content.out.split("\n")
        {
            let commit      = line.substring(0, line.find("=").unwrap());
            let mut parents = line.substring(line.find("=").unwrap()+1, line.len());
            let decreations = parents.substring(parents.find("=").unwrap()+1, parents.len());
            parents         = parents.substring(0, parents.find("=").unwrap());

            let mut parent_list: Vec<String> = vec![]; 
            for parent in parents.split(" ")
            {parent_list.push(parent.to_string());}

            commits_tree.insert(commit.to_string(), parent_list);
            commits.push(commit.to_string());

            if decreations != ""
            {
                for mut decreation in decreations.split(", ")
                {
                    if decreation.starts_with("tag: ")  
                    {
                        let tag: String = decreation.substring(5, decreation.len()).to_string();
                        if let Some(show_ref) = git::show_ref(&tag, remote_path)
                        {tags.insert(tag, ("".to_string(), show_ref));}
                        else {panic!("export, run: failed the ref");}
                    }
                    else                                
                    {
                        if decreation.starts_with("HEAD -> ")
                        {decreation = decreation.substring(decreation.find("->").unwrap()+3, decreation.len())}
                     
                        branches.insert(decreation.to_string(), ("".to_string(), commit.to_string()));
                    }
                }
            }
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

        for branch in export.get_many::<String>("Name").unwrap()
        {
            let commit_id: &str = &get_ref(branch, "branch", remote_path).0;
            commits.push(commit_id.to_string()); 

            let tree = find_commits (commit_id.to_string(), &rooted_commits, &visted, remote_path);

            for id in tree.keys()
            {commits_tree.insert(id.to_string(), tree[id].clone());}

            branches.insert(branch.to_string(), ("".to_string(), commit_id.to_string()));
        }
    }
    else if sub.subcommand_matches("Tag").is_some()
    {
        for tag in export.get_many::<String>("Name").unwrap()
        {
            let tag_ref: &str = &get_ref(tag, "tag", remote_path).0;
            tags.insert(tag.to_string(), ("".to_string(), tag_ref.to_string()));
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


    for tag in tags.keys()
    {
        let ref_id: &str   = &tags[tag].1;

        let tag_info = tag_commit(ref_id, remote_path);
        let pre: serde_json::Value;

        if tags.get(tag).unwrap().0 == ""  {pre = json!(null)}
        else        {pre = json!(tags.get(tag).unwrap().0)}

        
        if !tag_info.1
        {
            build_json["tag"]["light"].as_object_mut().unwrap().insert(tag.to_string(), json!({"previous": pre, "current": &ref_id}));


            if !commits_tree.contains_key(ref_id)
            {required_commits.push(ref_id.to_string());}
        }
        else
        {
            build_json["tag"]["annotated"].as_object_mut().unwrap().insert(tag.to_string(), json!({"previous": pre, "current": &ref_id}));
            
            export_objects.push(ref_id.to_string());

            if !commits_tree.contains_key(&tag_info.0)
            {required_commits.push(tag_info.0);}
        }
    }

    for tag in retired_tags.keys()
    {build_json["tag"]["retired"].as_object_mut().unwrap().insert(tag.to_string(), json!(retired_tags.get(tag).unwrap()));}

    for branch in branches.keys()
    {
        let commit: &String   = &branches.get(branch).unwrap().1;

        let pre: serde_json::Value;
        if &branches.get(branch).unwrap().0 == ""       {pre = json!(null)}
        else        {pre = json!(&branches.get(branch).unwrap().0)}

        build_json["branch"]["active"].as_object_mut().unwrap().insert(branch.to_string(), json!({"previous": pre, "current": commit}));
    }

    for branch in retired_branches.keys()
    {build_json["branch"]["retired"].as_object_mut().unwrap().insert(branch.to_string(), json!(retired_branches.get(branch).unwrap()));}

    if build_json["tag"]["light"].as_object().unwrap().is_empty() && build_json["tag"]["annotated"].as_object().unwrap().is_empty() && 
        build_json["tag"]["retired"].as_object().unwrap().is_empty() && build_json["branch"]["active"].as_object().unwrap().is_empty() && 
        build_json["branch"]["retired"].as_object().unwrap().is_empty()
    {
        println!("No Changes detected, refusing to make an export.");
        std::process::exit(0);
    }
    
    let mut seen: Vec<String> = vec![];

    println!("{:?} {:?}", commits, commits_tree);
    for commit in commits
    {
        seen.push(commit.clone());
        if commits_tree.contains_key(&commit)
        {export_objects.append(&mut scan_changes(&commit, &mut commits_tree, &mut required_commits, &seen, remote_path));}
    }

    build_json.insert("requiredCommits".to_string(), json!(required_commits));

    let json = Value::Object(build_json);
    let out: PathBuf = file_path("pack", time_stamp, "exp", out_path);
    let mut pack: String = "".to_string();
    
    if !export_objects.is_empty()
    { pack = pack_objects (&export_objects, remote_path); }

    fs::write(out, json.to_string() + "\n" + &pack).unwrap();
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
            if let Some(parents) = git::get_parents(&current_commit, path)
            {
                for parent in &parents
                {process_list.push(parent.to_string())}

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
fn scan_changes (commit: &str, commits: &mut HashMap <String, Vec<String>>,  required_commits: & mut Vec<String>, seen: & Vec<String>, path: & str) -> Vec<String> 
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
                    {changes.append(&mut pars_content (commit, &current_commit, path));}
                    else {changes.append(&mut pars_content (commit, "", path));}
                }
                else if parents.len() == 1
                {commits_queue.push(parents[0].to_string());}
                else 
                {
                    changes.append(&mut pars_content(&parent_commit, &parents[0], path));

                    for  parent in parents
                    {parents_queue.push(parent.to_string());}
                }

                commits.remove(&current_commit);
            }
            else if current_commit != parent_commit 
            {
                changes.append(&mut pars_content(&parent_commit, &current_commit, path)); 

                if !changes.contains(&current_commit) && !seen.contains(&current_commit)
                {required_commits.push(current_commit);}
            }
            else if!seen.contains(&current_commit)
            {required_commits.push(current_commit);}
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


/* pars_content
 *============================================
 * Purpose:     find objects that changed
 * Input:       commit, parent, path
 * Results:     array of content
 * Notes:       
 */
fn pars_content (commit: &str, parent: &str, path: &str) -> Vec<String> 
{
    let results = git::get_objs(commit, parent, path);
    if results == None
    {panic!("export, pars_content: failed to get objects "); }

    return results.unwrap();
}


/* pack_objects
 *============================================
 * Purpose:     produce a pack of objects need to be export
 * Input:       list of objects, remote path
 * Results:     encoded pack data
 * Notes:       
 */
fn pack_objects (objects: &Vec<String>, remote_path: &str)->String
{    
    let mut command = cmd::new("git");
    command.arg("pack-objects").arg("--stdout")
        .stdin(Stdio::piped()).stdout(Stdio::piped());

    if remote_path != ""
    {command.current_dir(remote_path);}

    let mut pack_cmd = command.spawn().unwrap();


    if let Some(ref mut stdin) = pack_cmd.stdin 
    {stdin.write_all(objects.join("\n").as_bytes()).unwrap();}

    let result = pack_cmd.wait_with_output().unwrap();

    if !result.status.success() 
    {panic!("export, pack_objects: failed to pack objects");}

    let content = result.stdout;

    return BASE64_STANDARD.encode(content);
}