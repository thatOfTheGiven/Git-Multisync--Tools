/* Git-Pack-Sync::git
 *============================================
 * Purpose:     git tools
 */

use substring::Substring;

use shared::cmd as cmd;


/* all_ref
 *============================================
 * Purpose:     return all tags and heads
 * Input:       path
 * Results:     Array of objects
 * Notes:       
 */
pub fn all_ref(path: &str) -> Option<Vec<String>>
{
    let output = cmd("git", ["show-ref", "--tags", "--heads"].to_vec(), &path);


    if output.status
    {return Some(output.out.split("\n").map(|s| s.to_string()).collect())}

    return None
}

/* show_ref
 *============================================
 * Purpose:     show the ref
 * Input:       object and path
 * Results:     ref
 * Notes:       
 */
pub fn show_ref(obj: &str, path: &str) -> Option<String>
{
    let output = cmd("git", ["show-ref", "-s", obj].to_vec(), &path);
    if output.status
    {return Some(output.out)}

    return None
}

/* list_branches
 *============================================
 * Purpose:     a list of branches
 * Input:       path
 * Results:     branch list
 * Notes:       
 */
pub fn list_branches(path: &str) -> Option<Vec<String>>
{
    let output = cmd("git", ["branch", "-l"].to_vec(), &path);


    if output.status
    {return Some(output.out.split("\n").map(|s| s.to_string().substring(2, s.len()).to_string()).collect())}

    return None
}

/* default_branch
 *============================================
 * Purpose:     get the default branch
 * Input:       path
 * Results:     branch
 * Notes:       
 */
pub fn default_branch(path: &str) -> Option<String>
{
    let output = cmd("git", ["remote", "show", "origin"].to_vec(), path); 
 
    if output.status
    {
        let mut primary: String = output.out.substring(output.out.find("HEAD").unwrap(), output.out.len()).to_string();
        primary = primary.substring(primary.find(":").unwrap()+2, primary.len()).to_string();
        primary = primary.substring(0, primary.find("\n").unwrap()).to_string();

        return Some(primary)
    }

    return None
}


/* list_tags
 *============================================
 * Purpose:     get a list of tags
 * Input:       Path
 * Results:     array of tags
 * Notes:       
 */
pub fn list_tags(path: &str) -> Option<Vec<String>>
{
    let output = cmd("git", ["tag", "-l"].to_vec(), &path);


    if output.status
    {return Some(output.out.split("\n").map(|s| s.to_string()).collect())}

    return None
}


/* run
 *============================================
 * Purpose:     get_commit
 * Input:       given tag or branch get commit
 * Results:     commit
 * Notes:       
 */
pub fn get_commit(obj: &str, path: &str) -> Option<String>
{
    let output = cmd("git", ["rev-list", "-n", "1", obj].to_vec(), &path);


    if output.status
    {return Some(output.out)}

    return None
}

/* get_parents
 *============================================
 * Purpose:     get the parents of a commit
 * Input:       commit path
 * Results:     array of parents
 * Notes:       
 */
pub fn get_parents(commit: &str, path: &str) -> Option<Vec<String>>
{
    let output = cmd("git", ["show", "-s", "--pretty=%P", commit].to_vec(), &path);


    if output.status
    {return Some(output.out.split("\n").map(|s| s.to_string()).collect())}

    return None
}

/* is_ansessor
 *============================================
 * Purpose:     test to see if a commit is an ancestor to another
 * Input:       commit, ancestor, path
 * Results:     true if is ancestor
 * Notes:       
 */
pub fn is_ancestor(commit: &str, ancestor: &str, path: &str) -> bool
{
    let results = cmd("git", ["merge-base", "--is-ancestor", ancestor, commit].to_vec(), path);

    return results.status
}

/* sha_type
 *============================================
 * Purpose:     run type for shai
 * Input:       sha, path
 * Results:     type
 * Notes:       
 */
pub fn sha_type(sha: &str, path: &str) -> Option<String>
{

    let output = cmd("git", ["cat-file", "-t", sha].to_vec(), &path);

    if output.status
    {return Some(output.out)}

    return None
}

/* get_objs
 *============================================
 * Purpose:     get objects that changes in a commit
 * Input:       commit, ancestor, path
 * Results:     vector of objects names 
 * Notes:       
 */
pub fn get_objs(commit: &str, ancestor: &str, path: &str) -> Option<Vec<String>>
{
    let arg: String; 
    if ancestor != "" {arg = ancestor.to_owned() + ".." + commit;}
    else {arg = commit.to_string()}

    let output = cmd("git", ["rev-list", "--objects", "--no-object-names", &arg ].to_vec(), path);
    if output.status
    {return Some(output.out.split("\n").map(|s| s.to_string()).collect())}

    return None
}

/* update_ref
 *============================================
 * Purpose:     update the ref for tags and branches
 * Input:       commit, ref_path, path
 * Results:     NONE
 * Notes:       
 */
pub fn update_ref(commit: &str, ref_path: &str, path: &str) -> bool
{
    let output = cmd("git", ["update-ref",  ref_path, commit].to_vec(), path);
    return output.status
}

/* delete_branch
 *============================================
 * Purpose:     delete branch
 * Input:       branch path
 * Results:     results
 * Notes:       
 */
pub fn delete_branch (branch: &str, path: &str) -> bool
{
    let output = cmd("git", ["branch",  "-D", branch].to_vec(), path);
    return output.status
}

// create_annotated_tag, is done by update_ref
/* create_light_tag
 *============================================
 * Purpose:     create a ligth tag
 * Input:       tag, commit, path
 * Results:     results
 * Notes:       
 */
pub fn create_light_tag (tag: &str, commit: &str, path: &str) -> bool
{
    let output = cmd("git", ["tag", tag, commit].to_vec(), path);
    return output.status
}

/* delete_tag
 *============================================
 * Purpose:     delete the tag
 * Input:       tag path
 * Results:     results
 * Notes:       
 */
pub fn delete_tag (tag: &str, path: &str) -> bool
{
    let output = cmd("git", ["tag",  "-d", tag].to_vec(), path);
    return output.status
}