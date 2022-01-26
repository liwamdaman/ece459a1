use std::cmp::Ordering;
use std::collections::VecDeque;
use rpkg::debversion::DebianVersionNum;
use crate::Packages;
use crate::packages::Dependency;

impl Packages {
    /// Computes a solution for the transitive dependencies of package_name; when there is a choice A | B | C, 
    /// chooses the first option A. Returns a Vec<i32> of package numbers.
    ///
    /// Note: does not consider which packages are installed.
    pub fn transitive_dep_solution(&self, package_name: &str) -> Vec<i32> {
        if !self.package_exists(package_name) {
            return vec![];
        }

        let deps : &Vec<Dependency> = &*self.dependencies.get(self.get_package_num(package_name)).unwrap();
        let mut dependency_set = vec![];

        // implement worklist
        dependency_set.append(&mut deps.iter().map(|x| x[0].package_num).collect());
        let mut prev_len = 0;
        while prev_len < dependency_set.len() {
            let mut new_deps = vec![];
            for i in prev_len..dependency_set.len() {
                let inner_deps = self.dependencies.get(&dependency_set[i]).unwrap();
                for inner_dep in inner_deps {
                    if !dependency_set.contains(&(inner_dep[0].package_num)) {
                        new_deps.push(inner_dep[0].package_num);
                    }
                }
            }
            // add new deps to set, repeat
            prev_len = dependency_set.len();
            dependency_set.append(&mut new_deps);
        }
        // No new dependencies added, return
        return dependency_set;
    }

    /// Computes a set of packages that need to be installed to satisfy package_name's deps given the current installed packages.
    /// When a dependency A | B | C is unsatisfied, there are two possible cases:
    ///   (1) there are no versions of A, B, or C installed; pick the alternative with the highest version number (yes, compare apples and oranges).
    ///   (2) at least one of A, B, or C is installed (say A, B), but with the wrong version; of the installed packages (A, B), pick the one with the highest version number.
    pub fn compute_how_to_install(&self, package_name: &str) -> Vec<i32> {
        if !self.package_exists(package_name) {
            return vec![];
        }
        let mut dependencies_to_add : Vec<i32> = vec![];

        // implement more sophisticated worklist
        let mut worklist: VecDeque<i32> = VecDeque::new();
        worklist.push_back(self.get_package_num(package_name).clone());
        while !worklist.is_empty() {
            let item = worklist.pop_front().unwrap();
            dependencies_to_add.push(item);
            let deps = self.dependencies.get(&item).unwrap();
            for dep in deps {
                match self.dep_is_satisfied(dep) {
                    None => {
                        if dep.len() > 1 {
                            let installed_alternatives_with_wrong_version = self.dep_satisfied_by_wrong_version(dep);
                            if installed_alternatives_with_wrong_version.len() == 1 {
                                // First case: only one alternative has a version installed
                                if !dependencies_to_add.contains(self.get_package_num(installed_alternatives_with_wrong_version[0])) && !worklist.contains(self.get_package_num(installed_alternatives_with_wrong_version[0])) {
                                    worklist.push_back(self.get_package_num(installed_alternatives_with_wrong_version[0]).clone())
                                }
                            } else {
                                /*
                                Second case: no alternatives with versions installed, or multiple alternatives with versions installed.
                                We should choose to add the package with the highest AVAILABLE version.
                                 */
                                let mut package_with_highest_available_version = &dep[0];
                                let mut highest_available_version = self.available_debvers.get(&dep[0].package_num).unwrap();
                                for alternative in dep {
                                    let available_version = self.available_debvers.get(&alternative.package_num).unwrap();
                                    if available_version.cmp(highest_available_version) == Ordering::Greater {
                                        package_with_highest_available_version = alternative;
                                        highest_available_version = available_version;
                                    }
                                }
                                if !dependencies_to_add.contains(&package_with_highest_available_version.package_num) && !worklist.contains(&package_with_highest_available_version.package_num) {
                                    worklist.push_back(package_with_highest_available_version.package_num);
                                }
                            }
                        } else {
                            // Only one alternative, add to worklist and move on
                            if !dependencies_to_add.contains(&dep[0].package_num) && !worklist.contains(&dep[0].package_num) {
                                worklist.push_back(dep[0].package_num);
                            }
                        }
                    }
                    Some(_) => {} // Doesn't need to be added to worklist, continue
                }
            }
        }

        // Remove the first element in the list (it will be the original package itself).
        dependencies_to_add.remove(0);
        return dependencies_to_add;
    }
}
