use std::io::BufRead;
use std::fs::read_dir;
use std::ffi::OsStr;
use std::io;

#[derive(Clone, Serialize)]
pub struct Skater {
    pub number: String,
    pub name: String,
}


#[derive(Clone, Serialize)]
pub struct Team {
    pub name: String,
    pub skaters: Vec<Skater>,
}

impl Team {
    fn from_file<R>(mut input: R)
                    -> io::Result<Team> where R : BufRead
    {

        fn invalid_data<T>(s: String) -> io::Result<T> {
            Err(io::Error::new(io::ErrorKind::InvalidData, s))
        }

        let mut teamname = String::new();
        match input.read_line(&mut teamname) {
            Ok(_) => teamname.pop(),
            Err(e) => return Err(e),
        };
        let mut ret = Team { name: teamname, skaters: Vec::new() };
        for l in input.split(b'\n') {
            let line = l?;
            let mut items = line.splitn(2, |c| *c == b'\t');
            let num = match items.next() {
                None => continue,
                Some(n) if n.len() == 0 => continue,
                Some(n) if n.len() > 4 =>
                    return invalid_data(format!("Skater number {:?} too long", n)),
                Some(n) => match String::from_utf8(Vec::from(n)) {
                    Err(_) => return invalid_data(format!("Bad skater name")),
                    Ok(na) => na,
                }
            };
            let name_vec = Vec::from(items.next().unwrap_or(b""));
            let name = match String::from_utf8(name_vec) {
                Err(e) => return invalid_data(format!("Invalid utf-8 {}", e)),
                Ok(s) => String::from(s),
            };

            ret.skaters.push(Skater { name: name, number: num });
        }

        ret.skaters.sort_by(|k1, k2| k1.number.cmp(&k2.number));
        Ok(ret)
    }
}

pub fn load_rosters(rosterdir: &OsStr) -> io::Result<Vec<Team>> {
    let mut rosters = Vec::new();
    for entry in read_dir(rosterdir)? {
        let path = entry?.path();
        let reader = io::BufReader::new(::std::fs::File::open(path)?);
        let team = Team::from_file(reader)?;
        println!("Loaded 1 roster");
        rosters.push(team);
    }
    Ok(rosters)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    #[test]
    fn test_fmt() {
        let mut stuff = b"Toaster City\n12\tBob Rodney\n34\tFred Fredney\n";
        let res = super::Team::from_file(Cursor::new(&stuff[..]));
        assert!(res.is_ok());
        let team = res.unwrap();
        assert_eq!(team.name, "Toaster City");
        assert_eq!(team.skaters[0].name, "Bob Rodney");
    }
}
