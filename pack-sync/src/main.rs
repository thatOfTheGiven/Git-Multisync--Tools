/* Git-Pack-Sync
 *============================================
 * Purpose:     Allow for an export, import, and inspection of git sync
 */

use chrono::{Utc, DateTime};
use std::path::{Path, PathBuf};
use clap::{ArgGroup, command, Arg, Command, ValueHint};



mod snapshot;
mod export;
mod import;
mod inspect;


fn main() 
{

    let cli = command!().arg_required_else_help(true) // requires `cargo` feature        
        .subcommand(Command::new("Snapshot").about("Record state of repo, to be used in a derivative sync")
            .arg(Arg::new("remote_path").help("Path to git repo")
                .short('C').value_hint(ValueHint::DirPath).required(false))
            .arg(Arg::new("OutPath").help("Directory or file file to contain the snapshot").value_name("Directory").required(false))
        )
        .subcommand(Command::new("Export").about("Run an export git objects")
            .arg_required_else_help(true) 
            .subcommand_precedence_over_arg(true)
            .arg(Arg::new("remote_path").help("Path to git repo")
                .short('C').value_hint(ValueHint::DirPath).global(true))
            .subcommand(Command::new("Snapshot").about("Uses a previously recorded snapshot file to find content to export")
                .arg(Arg::new("Input").help("Snapshot file to compare against")
                    .value_name("Snapshot file").required(true))
                .arg(Arg::new("primary").long("Primary").short('P').value_delimiter(',').help("Primary Branches to better extract content, defaults to default branch")
                    .value_name("Branch").required(false))
                .arg(Arg::new("OutPath").help("Directory to write the snapshot and export to").value_name("Directory").required(true))            
            )
            .subcommand(Command::new("Diff").about("Compares commits")
                .arg(Arg::new("root").long("Root").short('R').value_delimiter(',').help("The end point compare commits. Defaults to parent")
                    .value_name("Tag/Branch/Commit Id").required(false))                 
                .arg(Arg::new("Selection").value_delimiter(',').help("Selection to compare against").value_name("Tag/Branch/Commit Id").required(true))
                .arg(Arg::new("OutPath").help("Directory to write export").value_name("Path to export").required(true))                       
            )
            .subcommand(Command::new("Branch").about("Use Git Branch Id")                
                .arg(Arg::new("root").long("Root").short('R').value_delimiter(',').help("The root object to stop exporting")
                    .value_name("Tag/Branch/Commit Id").required(false)) 
                .arg(Arg::new("Name").value_delimiter(',').help("Set the Branch names")
                    .value_name("Branch Name").required(true))
                .arg(Arg::new("OutPath").help("Path to write the export").value_name("Path to export").required(true))            
            )
            .subcommand(Command::new("Tag").about("Export Tag")
                .arg(Arg::new("Name").value_delimiter(',').help("Set the Tag names")
                    .value_name("Tag Name").value_delimiter(',').required(true))
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
            .arg(Arg::new("objects").help("Select what git object to import.")
                .long("importing").value_name("Rule")
                .value_parser(["None", "Tags", "Branches", "Both"]).default_value("Both"))
            .arg(Arg::new("retired").help("Select what git object to delete when retired.")
                .long("retiring").value_name("Rule")
                .value_parser(["None", "Tags", "Branches", "Both"]).default_value("Both"))
            .arg(Arg::new("overwrite").help("Import overwrite rules.")
                .short('o').long("overwrite").value_name("Rule")
                .value_parser(["Hard", "Mixed", "Soft"]).default_value("Mixed"))
            .arg(Arg::new("include").help("Include branch by pattern or exact name")
                .short('i').long("include")
                .value_name("Exact/Pattern")
                .value_delimiter(','))
            .arg(Arg::new("exclude").help("Exclude branch by pattern or exact namee")
                .short('e').long("exclude")
                .value_name("Exact/Pattern")
                .value_delimiter(','))            
            .arg(Arg::new("import").help("Exported Object to import")
                .required(true))
        )
        .subcommand(Command::new("Inspect").about("Inspect export content")
            .arg_required_else_help(true) 
            .args_conflicts_with_subcommands(true)
            .subcommand_precedence_over_arg(true)
            .subcommand(Command::new("Json").about("JSON data from export")
                .arg(Arg::new("import").help("Exported Object")
                    .required(true))
                .arg(Arg::new("outfile").help("Extract data into")
                    .short('o').required(false))                
            )
            .subcommand(Command::new("Pack").about("extracts the pack file")
                .arg(Arg::new("import").help("Exported Object")
                    .required(true))
                .arg(Arg::new("outfile").help("Extract data into")
                    .short('o').required(false))
            )
            .subcommand(Command::new("Branches").about("list the branches in the export")
                .arg(Arg::new("format").help("formatted output").short('f')
                    .value_parser(["Long", "Index", "Event", "Simple"]).default_value("Event"))
                .arg(Arg::new("status").help("filter output based on status").short('s')
                    .value_parser(["Active", "Retired", "All"]).default_value("Active"))
                .arg(Arg::new("import").help("Exported Object to import")
                    .required(true))
                .arg(Arg::new("outfile").help("Extract data into")
                    .short('o').required(false))
            )
            .subcommand(Command::new("Tags").about("List the tags in the export")
                .arg(Arg::new("format").help("formatted output").short('f')
                    .value_parser(["Long", "Index", "Event", "Simple"]).default_value("Event"))
                .arg(Arg::new("status").help("filter output based on status").short('s')
                    .value_parser(["Active", "Retired", "All"]).default_value("Active"))
                .arg(Arg::new("type").help("filter output based on type").short('t')
                    .value_parser(["Light", "Annotated", "All"]).default_value("All"))
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
                out_path = shared::file_path("pack", &time_stamp, "snp", path);
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