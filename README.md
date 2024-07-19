
# Git Multi-Site/Sync Plumbing

Provide the plumbing tools to allow for a disconnected Multi-Site/Sync. 
This is not a turn key tooling.


## Alternative tooling
As of writing there are three options other then the one I built. I feel the need to explain why they do not fit for purpose. Also none of these tools provide a means to send lfs files.

#### Cloning
Cloning is a far solution so long as your repos are small. However you will be duplicating all content in every export and as the source grows so does the time and space need to sync. 

#### Patch
The Patch tool works well for it designed and from a high level seems to be the right answer. It designed to be a tool to export changes from a fork project back to main via email. The issue is that Patch has two flaws, first it cannot properly handle merged commits (see [git-format-patch](https://git-scm.com/docs/git-format-patch) CAVEATS sections). The first flaw can be overcome by exporting each commit as it own patch, which would also require you to import each in correct order and manually rebuild the commit tree. The Second flaw is that any generated commits from a patch would have a new commit id, which in turn would be replace when a cloned import from site is completed. The second flaw will cause havoc to developers who need to use changes in the sync site, but write then in the source.

#### Bundle
Bundle tool, is another tool that shows great promise. It uses a ref info to determin what content to export. Like my tool it uses the git pack-objects tools to provide the export. This allows the source and sync sites to have the exact same content, and no commit Ids would be wrong. The issue comes down when trying to sync Developers topic branches, the only meathead that bundle provides is a timestamp concept. Timestamps works off of created time on a commit. If a commit is not sync to the central repo till after the timestamp window, it will not sync. Furthermore if a new commit whos parent was not sent will cause the full import to fail. This will cause additional delays as the administrator sync the missing content.
## A word to Security
This tool is complete written in Rust, and use git native tools and lfs tools.

#### Export files (exp)
All content is sent in a human readable formats. The first line is json content describing the export. Any additional lines in the export is the content being sent to sync site. In the case of Pack exports the line 2 is a base64 encoding of a pack file, while lfs exports line 2 and on started with the hash name followed by a ":" deliminator and a base64 encoding of the file content.

#### Snapshot files (snp)
It is a human reable file recording the state of git repo or lfs 

#### Sensitive content
If sensitive content is discovered, I recommend deleting any all export files that may contain the sensitive data. If that sensitive data exists in name of a branch or tag you will also want to delete the snapshot files. Once deletion is done I would clean up the source site and then do a full export (clone) from the source to the sync.

#### Usefull Tools
- [index-pack](https://git-scm.com/docs/git-index-pack)
- [verify-pack](https://git-scm.com/docs/git-verify-pack)
- [pack-objects](https://git-scm.com/docs/git-pack-objects)
- [unpack-objects](https://git-scm.com/docs/git-unpack-objects)
- [lfs](https://github.com/git-lfs/git-lfs/tree/main)
- [base64 wiki](https://en.wikipedia.org/wiki/Base64)
- [base64 man page](https://linux.die.net/man/1/base64)
- [rust](https://www.rust-lang.org)
## Recommend Procedure
### Intal/Full Sync
    note you may run into protections by your centerized repo at sync site
#### Source Site
    1. Run a clone with the --mirror flag (keep repo)
    2. tar/zip and ship it to sync site
    3. run pack-sync Snapshot (keep Snapshot)
    4. (Optional only do if repo use lfs) run: git-lfs-sync Export All, ship export (keep Snapshot)

#### Sync Site
    5. create centralized git repo for the sync site
    6. expanded the repo from source (keep repo)
    7. set the repo origin to the centralized git repo
    8. (Optional only do if repo use lfs) run: git-lfs-sync Import
    9. git push origin --all
    10. git push origin --tags


### Increment/Snapshot Sync
#### Source Site
    1. Run git fetch
    2. run pack-sync Export Snapshot, ship export (keep Snapshot)
    3. (Optional only do if repo use lfs) run: lfs-sync Export Snapshot, ship export (keep Snapshot)

#### Sync Sit
    4. run: lfs-sync Import
    5. run: pack-sync Import
    6. git push origin --all
    7. git push origin --tags    

## Run
### pack-sync
#### Snapshot
```bash
  pack-sync Snapshot [Options] <Path>
  Purpose: to record the state of the repo
  Args:
    <Path>   Path to write out state of the repo

  Options:
    -C <repo>   Path to git repo
```

#### Export
```bash
  pack-sync Export Snapshot [Options] <Snapshot file> <Path>
  Purpow: To Export Snapshot of the commit
  Args:
    <Path>   Path to write out state of the repo
    <Snapshot file>   Path to snapshot file

  Options:
    -C <repo>   Path to git repo
    -P --primary <Tag/Branch/Commit Id>  Primary commit/branch/tag, to limit export


  pack-sync Export Diff [Options] <Tag/Branch/Commit Id> <Path>
  Purpose: either to export commit or range of commits
  Args:
    <Path>       Path to write out state of the repo
    <Tag/Branch/Commit Id>  Commit to export

  Options:
    -C <repo>   Path to git repo
    -R, --Root <Tag/Branch/Commit Id>  The root object to stop exporting


  pack-sync Export Branch [Options] <Branch> <Path>
  Purpose: to export all commits contain in branch
  Args:
    <Path>       Path to write out state of the repo
    <Branch>     branch to export

  Options:
    -C <repo>   Path to git repo
    -R, --Root <Tag/Branch/Commit Id>  The root object to stop exporting


  pack-sync Export Tag [Options] <Tag> <Path>
  Purpose: to export a tag
  Args:
    <Path>       Path to write out state of the repo
    <Tag>         branch to export

  Options:
    -C <repo>   Path to git repo
    

  pack-sync Export Timestamp [Options] <--Date <YYYY-MM-DD> | --Time <HH:MM:SS>> <Path>
  Purpose: to export all commits contain in branch
  Args:
    <Path>       Path to write out state of the repo

  Options:
    -C <repo>   Path to git repo
    -D, --Date <YYYY-MM-DD>  Set the Date to export
    -T, --Time <HH:MM:SS>    Set the Time to export
```
#### Inspect
```bash
  pack-sync Inspect Pack [Options] <import>
  Purpose: make the pack file available
  Args:
    <Input>       Input file

  Options:
    -o <outfile>   Extract pack into either directory or file


  pack-sync Inspect Json [Options] <import>
  Purpose: make the pack file available
  Args:
    <Input>       Input file

  Options:
    -o <outfile>   Extract join into either directory or file


  pack-sync Inspect Branches [Options] <import>
  Purpose: make the Branch data available
  Args:
    <Input>       Input file

  Options:
    -f <format>     formatted output [default: Event] 
                            [Long, Index, Event, Simple]
    -s <status>       filter output based on status [default: Active] 
                            [Active, Retired, All]
    -o <outfile>      Extract data into


  pack-sync Inspect Tags [Options] <import>
  Purpose: make the Tag data available
  Args:
    <Input>       Input file

  Options:
    -f <format>     formatted output [default: Event] 
                            [Long, Index, Event, Simple]
    -s <status>       filter output based on status [default: Active] 
                            [Active, Retired, All]
    -t <type>         filter output based on type [default: All] 
                            [Light, Annotated, All]
    -o <outfile>      Extract data into
```

#### Import
```bash
  pack-sync Import [Options] <import>
  Purpose: import sync
  Args:
    <Input>       Input file

  Options:
    -C <repo>               Path to git repo
    --importing <Rule>      Importing type  [default: Both]
                                 [None, Tags, Branches, Both]
    --retiring <Rule>       Retires type  [default: Both]
                                 [None, Tags, Branches, Both]
    --overwrite <Rule>      overwrite Rules [default: Mixed]
                                 [Soft, Mixed, Hard]
                        Hard: will Always delete or always update
                        Mixed: will Always delete and update only if commit is expected or is an ansessor 
                        Soft: Will only replace if commit is exptected
    -i --include <Exact/Pattern>  Run against branch that match include
    -e --exclude <Exact/Pattern>  Run against branch that do notmatch include
```



### lfs-sync
#### Snapshot
```bash
  lfs-sync Snapshot [Options] <Path>
  Purpose: to record the state of the repo
  Args:
    <Path>   Path to write out state of the repo

  Options:
    -C <repo>   Path to git repo
```

#### Export
```bash
  lfs-sync Export All [Options] <Path>
  Purpow: To Export all lfs files
  Args:
    <Path>   Path to write out state of the repo

  Options:
    -C <repo>   Path to git repo


  lfs-sync Export Snapshot [Options] <Snapshot file> <Path>
  Purpow: To Export Snapshot lfs files
  Args:
    <Path>   Path to write out state of the repo
    <Snapshot file>   Path to snapshot file

  Options:
    -C <repo>   Path to git repo
    -P --primary <Tag/Branch/Commit Id>  Primary commit/branch/tag, to limit export


    lfs-sync Export files [Options] <File Path> <Path>
  Purpose: either  to export lfs based on file path
  Args:
    <Path>       Path to write out state of the repo
    <File>       File path

  Options:
    -C <repo>   Path to git repo


  lfs-sync Export Diff [Options] <Tag/Branch/Commit Id> <Path>
  Purpose: either  to export lfs based on changes for commit or range of commits
  Args:
    <Path>       Path to write out state of the repo
    <Tag/Branch/Commit Id>  Commit to export

  Options:
    -C <repo>   Path to git repo
    -R, --Root <Tag/Branch/Commit Id>  The root object to stop exporting


  lfs-sync Export Branch [Options] <Branch> <Path>
  Purpose: to export all lfs based on changes contain on branch
  Args:
    <Path>       Path to write out state of the repo
    <Branch>     branch to export

  Options:
    -C <repo>   Path to git repo
    -R, --Root <Tag/Branch/Commit Id>  The root object to stop exporting

   

  lfs-sync Export Timestamp [Options] <--Date <YYYY-MM-DD> | --Time <HH:MM:SS>> <Path>
  Purpose: to export all lfs changes based since a givem timestamp
  Args:
    <Path>       Path to write out state of the repo

  Options:
    -C <repo>   Path to git repo
    -D, --Date <YYYY-MM-DD>  Set the Date to export
    -T, --Time <HH:MM:SS>    Set the Time to export
```

#### Inspect
```bash
  lfs-sync Inspect Content [Options] <import>
  Purpose: make the Content available
  Args:
    <Input>       Input file

  Options:
    -o <outfile>   Extract pack into either directory or file
    -b             Recreate the strcuture


  pack-sync Inspect Json [Options] <import>
  Purpose: make theJsonavailable
  Args:
    <Input>       Input file

  Options:
    -o <outfile>   Extract join into either directory or file


  lfs-sync Inspect Json [Options] <import>
  Purpose: make the Json available
  Args:
    <Input>       Input file

  Options:
    -o <outfile>   Extract pack into either directory or file


  lfs-sync Inspect Files [Options] <import>
  Purpose: make the Fiels data available
  Args:
    <Input>       Input file

  Options:
    -f <format>     formatted output [default: Simple] 
                            [Long, Simple]
    -o <outfile>      Extract data into


  lfs-sync Inspect Hash [Options] <import>
  Purpose: make the Hash data available
  Args:
    <Input>       Input file

  Options:
    -f <format>     formatted output [default: Simple] 
                            [Long, Simple]
                            [Light, Annotated, All]
    -o <outfile>      Extract data into
```

#### Import
```bash
  lfs-sync Import [Options] <import>
  Purpose: import sync
  Args:
    <Input>       Input file

  Options:
    -C <repo>               Path to git repo
```
