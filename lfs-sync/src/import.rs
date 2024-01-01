/* Git-lfs-Sync::import
 *============================================
 * Purpose:     Perform an import of pack files
 */

use std::path::PathBuf;
use std::env;
use std::path::Path;
use clap::ArgMatches;


use crate::inspect::extract_content;
//use crate::lfs;


/* run
 *============================================
 * Purpose:     run an import
 * Input:       ArgMAtches
 * Results:     NONE
 * Notes:       
 */
pub fn run(sub: &ArgMatches) 
{
    let mut lfs_import: PathBuf;

    if let Some(remote) = sub.get_one::<String>("remote_path")
    {lfs_import = Path::new(remote).to_path_buf();}
    else
    {lfs_import =  env::current_dir().unwrap();}


    if lfs_import.join(".git").exists()
    {lfs_import = lfs_import.join(".git")}

    lfs_import = lfs_import.join("lfs").join("objects");

    let _ = std::fs::create_dir_all(lfs_import.clone());
    let import: &str = sub.get_one::<String>("import").unwrap();

    extract_content(import, &true, Some(&lfs_import));
}