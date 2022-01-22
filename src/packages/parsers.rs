use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use regex::Regex;

use crate::Packages;
use crate::packages::RelVersionedPackageNum;

use rpkg::debversion;

const KEYVAL_REGEX : &str = r"(?P<key>(\w|-)+): (?P<value>.+)";
const PKGNAME_AND_VERSION_REGEX : &str = r"(?P<pkg>(\w|\.|\+|-)+)( \((?P<op>(<|=|>)(<|=|>)?) (?P<ver>.*)\))?";

impl Packages {
    /// Loads packages and version numbers from a file, calling get_package_num_inserting on the package name
    /// and inserting the appropriate value into the installed_debvers map with the parsed version number.
    pub fn parse_installed(&mut self, filename: &str) {
        let kv_regexp = Regex::new(KEYVAL_REGEX).unwrap();
        if let Ok(lines) = read_lines(filename) {
            let mut current_package_num = 0;
            for line in lines {
                if let Ok(ip) = line {
                    // do something with ip
                    if kv_regexp.is_match(&ip) {
                        let caps = kv_regexp.captures(&ip).unwrap();
                        let key = caps.name("key").unwrap().as_str();
                        let value = caps.name("value").unwrap().as_str();
                        //println!("{}: {}", key, value);

                        if key == "Package" {
                            current_package_num = self.get_package_num_inserting(&value);
                            //println!("Package: {}, package num: {}", value, current_package_num);
                        }

                        if key == "Version" {
                            let debver = value.trim().parse::<debversion::DebianVersionNum>().unwrap();
                            self.installed_debvers.insert(current_package_num, debver); // Assume we always receive Package line before the Version line
                        }
                    }
                }
            }
            // for (key, value) in &self.installed_debvers {
            //     println!("{}: {}", key, value);
            // }
        }
        println!("Packages installed: {}", self.installed_debvers.keys().len());
    }

    /// Loads packages, version numbers, dependencies, and md5sums from a file, calling get_package_num_inserting on the package name
    /// and inserting the appropriate values into the dependencies, md5sum, and available_debvers maps.
    pub fn parse_packages(&mut self, filename: &str) {
        let kv_regexp = Regex::new(KEYVAL_REGEX).unwrap();
        let pkgver_regexp = Regex::new(PKGNAME_AND_VERSION_REGEX).unwrap();

        if let Ok(lines) = read_lines(filename) {
            let mut current_package_num = 0;
            for line in lines {
                if let Ok(ip) = line {
                    // do more things with ip
                }
            }
        }
        println!("Packages available: {}", self.available_debvers.keys().len());
    }
}


// standard template code downloaded from the Internet somewhere
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
