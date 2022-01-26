use rpkg::debversion;
use rpkg::debversion::{DebianVersionNum, VersionRelation};
use crate::Packages;
use crate::packages::Dependency;

impl Packages {
    /// Gets the dependencies of package_name, and prints out whether they are satisfied (and by which library/version) or not.
    pub fn deps_available(&self, package_name: &str) {
        if !self.package_exists(package_name) {
            println!("no such package {}", package_name);
            return;
        }
        println!("Package {}:", package_name);
        // some sort of for loop...
        let dependencies = self.dependencies.get(self.get_package_num(package_name)).unwrap();
        for dep in  dependencies {
            println!("- dependency {:?}", self.dep2str(dep));
            match self.dep_is_satisfied(dep) {
                None => {
                    println!("-> not satisfied");
                }
                Some(package_name) => {
                    println!("+ {} satisfied by installed version {}", package_name, self.installed_debvers.get(self.get_package_num(package_name)).unwrap());
                }
            }
        }
    }

    /// Returns Some(package) which satisfies dependency dd, or None if not satisfied.
    pub fn dep_is_satisfied(&self, dd:&Dependency) -> Option<&str> {
        // presumably you should loop on dd
        for alternative in dd {
            if self.installed_debvers.contains_key(&alternative.package_num) {
                match &alternative.rel_version {
                    None => {
                        // Dependency has no version requirement, so any installed version of the dependency is satisfactory
                        return Some(self.get_package_name(alternative.package_num));
                    }
                    Some((required_version_relation, required_version)) => {
                        // Dependency has version requirement, compare versions
                        let required_v = required_version.parse::<debversion::DebianVersionNum>().unwrap();
                        let installed_v = self.installed_debvers.get(&alternative.package_num).unwrap();
                        if debversion::cmp_debversion_with_op(&required_version_relation, installed_v, &required_v) {
                            return Some(self.get_package_name(alternative.package_num));
                        }
                        // Else, move on to next alternative
                    }
                }
            }
        }
        return None;
    }

    /// Returns a Vec of packages which would satisfy dependency dd but for the version.
    /// Used by the how-to-install command, which calls compute_how_to_install().
    pub fn dep_satisfied_by_wrong_version(&self, dd:&Dependency) -> Vec<&str> {
        assert! (self.dep_is_satisfied(dd).is_none());
        let mut result = vec![];
        // another loop on dd
        for alternative in dd {
            if self.installed_debvers.contains_key(&alternative.package_num) {
                // We can assume this means that the dependency has a version requirement that is not met by the installed version, no need to check
                result.push(self.get_package_name(alternative.package_num));
            }
            // No versions have been installed yet for this alternative, do not add to result
        }
        return result;
    }
}

