/* lfs-Sync
 *============================================
 * Purpose:     Allow for an export, import, and inspection of git sync
 */

use chrono::{Utc, DateTime};
use std::path::{Path, PathBuf};
use clap::{ArgGroup, command, Arg, Command, ValueHint};


mod snapshot;
mod export;
mod inspect;
mod import;



fn main() 
{

    let cli = command!().arg_required_else_help(true) // requires `cargo` feature        
        .subcommand(Command::new("Snapshot").about("Record state of lfs, to be used in a derivative sync")
            .arg(Arg::new("remote_path").help("Path to git repo")
                .short('C').value_hint(ValueHint::DirPath).required(false))
            .arg(Arg::new("OutPath").help("Path to write out state of the repo").value_name("Path").required(false))
        )
        .subcommand(Command::new("Export").about("Run an export git objects")
            .arg_required_else_help(true) 
            .subcommand_precedence_over_arg(true)
            .arg(Arg::new("remote_path").help("Path to git repo")
                .short('C').value_hint(ValueHint::DirPath).global(true))
            .subcommand(Command::new("All").about("export all lfs")
                .arg(Arg::new("OutPath").help("Path to write out state of the repo and export").value_name("Path to export").required(true))            
            )
            .subcommand(Command::new("Snapshot").about("Uses a previously recorded snapshot file to find content to export")
                .arg(Arg::new("Input").help("Snapshot file to compare against")
                    .value_name("Snapshot file").required(true))
                .arg(Arg::new("OutPath").help("Path to write out state of the repo and export").value_name("Path to export").required(true))            
            )
            .subcommand(Command::new("Files").about("use a list files to export lfs content")
                .arg(Arg::new("Files").value_delimiter(',').help("List of files")
                    .value_name("file path").required(true))
                .arg(Arg::new("OutPath").help("Path to write out state of the repo and export").value_name("Path to export").required(true))            
            )
            .subcommand(Command::new("Diff").about("Compares commits")
                .arg(Arg::new("root").long("Root").short('R').value_delimiter(',').help("The root object to stop exporting")
                    .value_name("Tag/Branch/Commit Id").required(false))                 
                .arg(Arg::new("Selection").value_delimiter(',').help("Selection to compare against").value_name("Tag/Branch/Commit Id").required(true))
                .arg(Arg::new("OutPath").help("Path to write the export").value_name("Path to export").required(true))                       
            )
            .subcommand(Command::new("Branch").about("Use Git Branch Id")                
                .arg(Arg::new("root").long("Root").short('R').value_delimiter(',').help("The root object to stop exporting")
                    .value_name("Tag/Branch/Commit Id").required(false)) 
                .arg(Arg::new("Name").value_delimiter(',').help("Set the Branch names")
                    .value_name("Branch Name").required(true))
                .arg(Arg::new("OutPath").help("Path to write the export").value_name("Path to export").required(true))            
            )
            .subcommand(Command::new("Timestamp").about("Export all content since date and time")
                .arg(Arg::new("Date").help("Set the Date to export")
                    .short('D').long("Date").value_name("YYYY-MM-DD"))
                .arg(Arg::new("Time").help("Set the Time to export ")
                    .short('T').long("Time").value_name("HH:MM:SS"))
                .arg(Arg::new("OutPath").help("Path to write the export").value_name("Path to export").required(true))            
                .group(ArgGroup::new("vers").args(["Date", "Time"])
                    .multiple(true).required(true))
            )
        )
        .subcommand(Command::new("Import").about("Imports Git Objects")
            .args_conflicts_with_subcommands(true)
            .subcommand_precedence_over_arg(true)
            .arg(Arg::new("remote_path").help("Path to git repo")
                .short('C').value_hint(ValueHint::DirPath).global(true))
            .arg(Arg::new("import").help("Exported Object to import")
                .required(true))
        )
        .subcommand(Command::new("Inspect").about("Inspect export content")
            .arg_required_else_help(true) 
            .args_conflicts_with_subcommands(true)
            .subcommand_precedence_over_arg(true)            
            .subcommand(Command::new("Content").about("extracts the Content")
                .arg(Arg::new("import").help("Exported Object")
                    .required(true))
                .arg(Arg::new("outfile").help("Extract data into").required(true))
                .arg(Arg::new("build_structure").help("Build the structure of lfs")
                    .short('b').required(false).action(clap::ArgAction::SetTrue))
            )
            .subcommand(Command::new("Json").about("JSON data from export")
                .arg(Arg::new("import").help("Exported Object")
                    .required(true))
                .arg(Arg::new("outfile").help("Extract data into")
                    .short('o').required(false))                
            )
            .subcommand(Command::new("Files").about("list the files in the export")
                .arg(Arg::new("format").help("formatted output").short('f')
                    .value_parser(["Long", "Simple"]).default_value("Simple"))
                .arg(Arg::new("import").help("Exported Object to import")
                    .required(true))
                .arg(Arg::new("outfile").help("Extract data into")
                    .short('o').required(false))
            )
            .subcommand(Command::new("Hash").about("List the hash in the export")
                .arg(Arg::new("format").help("formatted output").short('f')
                    .value_parser(["Long", "Simple"]).default_value("Simple"))
                .arg(Arg::new("import").help("Exported Object to import") 
                    .required(true))
                .arg(Arg::new("outfile").help("Extract data into")
                    .short('o').required(false))
            )
        ).get_matches();

    


    if let Some(sub) = cli.subcommand_matches("Import")
    {import::run(sub) }
    else if let Some(sub) = cli.subcommand_matches("Inspect")
    {inspect::run(sub)}
    else 
    {
        let now: DateTime<Utc> = Utc::now();
        let time_stamp: String  = now.format("%Y%m%d-%H%M%S").to_string();

        if let Some(sub) = cli.subcommand_matches("Snapshot")
        {
            let mut out: Option<&Path> = None;
            let out_path: PathBuf;
            if let Some(path) = sub.get_one::<String>("OutPath") 
            {
                out_path = shared::file_path("lfs", &time_stamp, "snp", path);
                out = Some(&out_path);
            }

            let mut remote_path: String = "".to_string();
            if let Some(path) = sub.get_one::<String>("remote_path")
            {remote_path  = path.to_string()}

            snapshot::gen(&remote_path, out);
        }
        else if let Some(sub) = cli.subcommand_matches("Export")
        {export::run(sub, &time_stamp, now);}        
    }
}



/*
git lfs ls-files -a

git lfs fetch --all

git lfs push --all origin
*/