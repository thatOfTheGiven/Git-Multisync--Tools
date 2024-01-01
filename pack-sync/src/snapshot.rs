/* Git-Pack-Sync::snapshot
 *============================================
 * Purpose:     produce a snapshot information
 */

use std::path::Path;
use std::fs;

use git::all_ref;



/* gen
 *============================================
 * Purpose:     generate a snapshot of repo
 * Input:       remote path, out path
 * Results:     NONE
 * Notes:       
 */
pub fn gen(remote_path: &str, out_file: Option<&Path>)
{
    if let Some(content) = all_ref(remote_path)
    {
        if let Some(out) = out_file
        {fs::write(out, content.join("\n")).expect("Unable to write file");}
        else {println!("{}", content.join("\n"))}
    }
    else {panic!("snapshot, gen: Failed to get references.\n")}    
}