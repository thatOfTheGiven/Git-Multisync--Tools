/* shared
 *============================================
 * Purpose:     use shared functions
 */

use std::path::PathBuf;
use std::path::Path;
use std::process::Command as cmd;

pub struct Outcome {
    pub status: bool,
    pub out: String,
    pub error: String
}



/* cmd
 *============================================
 * Purpose:     run a command
 * Input:       execute, args, path
 * Results:     Outcome
 * Notes:       
 */
pub fn cmd(run: &str, args: Vec<&str>, path: &str) -> Outcome
{
    let mut command = cmd::new(run);
    command.args(args);

    if path != ""
    { command.current_dir(path); }

    let output = command.output().expect(&("Command failed: ".to_owned() + run));
    let mut out_content = String::from_utf8(output.stdout).unwrap();

    if out_content.ends_with("\n")
    {out_content = out_content.strip_suffix("\n").unwrap().to_string()}

    if out_content.ends_with("\r")
    {out_content = out_content.strip_suffix("\r").unwrap().to_string()}


    let mut err_content = String::from_utf8(output.stderr).unwrap();

    if err_content.ends_with("\n")
    {err_content = err_content.strip_suffix("\n").unwrap().to_string()}

    if out_content.ends_with("\r")
    {err_content = err_content.strip_suffix("\r").unwrap().to_string()}

    
    return Outcome{status: output.status.success(), out: out_content, error: err_content}
}

/* file_path
 *============================================
 * Purpose:     produce a path to write to
 * Input:       ePrfix, timestamp, extension, root
 * Results:     NONE
 * Notes:       
 */
pub fn file_path(prefix: &str, time_stamp: &str, extension: &str, root: &str) -> PathBuf
{
    let out_path: &Path = Path::new(&root);
    let out_file: PathBuf;

    if !out_path.exists() 
    {
        if !out_path.parent().unwrap().exists()
        {panic!("libs, creat_file: folder {:?} does not exists", out_path.to_str());}
        out_file = out_path.to_path_buf()
    }
    else if out_path.is_dir()
    {
        let mut prefix_input: String = "".to_string();
        if prefix != ""
        {prefix_input = prefix.to_owned() + "."}
        out_file = out_path.join(prefix_input.to_owned() + time_stamp + "." + extension);
    }
    else 
    {panic!("libs, creat_file: {:?} is a file", out_path);}

    return out_file; 
}