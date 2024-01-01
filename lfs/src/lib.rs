/* Git-Pack-Sync::lfs
 *============================================
 * Purpose:     git lfs tools
 */

use shared::cmd as cmd;


/* list
 *============================================
 * Purpose:     return all tags and heads
 * Input:       path
 * Results:     Array of objects
 * Notes:       
 */
pub fn list(path: &str) -> Option<Vec<String>>
{
    let output = cmd("git", ["lfs", "ls-files", "-al"].to_vec(), &path);

    if output.status
    {return Some(output.out.split("\n").map(|s| s.to_string()).collect())}

    return None
}

/* fetch
 *============================================
 * Purpose:     show the ref
 * Input:       object and path
 * Results:     ref
 * Notes:       
 */
pub fn fetch(path: &str) -> bool
{
    let output = cmd("git", ["lfs", "fetch", "--all"].to_vec(), &path);
    return output.status
}

/* push
 *============================================
 * Purpose:     a list of branches
 * Input:       path
 * Results:     branch list
 * Notes:       
 */
pub fn push(remote: &str, path: &str) -> bool
{
    let output = cmd("git", ["lfs", "push", "--all", remote].to_vec(), &path);
    return output.status
}
