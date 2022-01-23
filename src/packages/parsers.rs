use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use regex::{Captures, Match, Regex};

use crate::Packages;
use crate::packages::{Dependency, RelVersionedPackageNum};

use rpkg::debversion;
use rpkg::debversion::VersionRelation;

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
                    if kv_regexp.is_match(&ip) {
                        match kv_regexp.captures(&ip) {
                            None => {}
                            Some(caps) => {
                                let (key, value) = (
                                    caps.name("key").unwrap().as_str(),
                                    caps.name("value").unwrap().as_str()
                                );
                                match key {
                                    "Package" => {
                                        current_package_num = self.get_package_num_inserting(&value);
                                    },
                                    "Version" => {
                                        let debver = value.trim().parse::<debversion::DebianVersionNum>().unwrap();
                                        self.available_debvers.insert(current_package_num, debver); // Assume we always receive Package line before the Version line
                                    },
                                    "MD5sum" => {
                                        self.md5sums.insert(current_package_num, String::from(value));
                                    },
                                    "Depends" => {
                                        let dependencies = value.split(",");
                                        let mut dependencies_vec = Vec::new();
                                        for dependency in dependencies {
                                            let alternatives = dependency.split("|");
                                            let mut alternatives_vec = Vec::new();
                                            for alternative in alternatives {
                                                match pkgver_regexp.captures(alternative) {
                                                    None => {}
                                                    Some(caps) => {
                                                        // Assume that regex capture will always have "pkg", but not necessarily have "op" and "ver".
                                                        let package_num = self.get_package_num_inserting(caps.name("pkg").unwrap().as_str());
                                                        let mut rel_version = Option::None;
                                                        match caps.name("op") {
                                                            None => {}
                                                            Some(op) => {
                                                                let op: debversion::VersionRelation = op.as_str().parse::<debversion::VersionRelation>().unwrap();
                                                                // Assume that if regex captures on op, we will capture ver as well.
                                                                let ver: String = caps.name("ver").unwrap().as_str().to_string();
                                                                rel_version = Option::Some((op, ver));
                                                            }
                                                        }
                                                        let rel_versioned_package_num = RelVersionedPackageNum {
                                                            package_num,
                                                            rel_version
                                                        };
                                                        alternatives_vec.push(rel_versioned_package_num);
                                                    }
                                                }
                                            }
                                            dependencies_vec.push(alternatives_vec);
                                        }
                                        self.dependencies.insert(current_package_num, dependencies_vec);
                                    },
                                    _ => {}
                                }
                            }
                        }
                    }
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
