use urlencoding::encode;

use curl::easy::{Easy2, Handler, WriteError};
use curl::multi::{Easy2Handle, Multi};
use std::collections::HashMap;
use std::hash::Hash;
use std::time::Duration;
use std::str;
use std::sync::atomic::{AtomicI32, Ordering};

use crate::Packages;

struct Collector(Box<String>);
impl Handler for Collector {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        (*self.0).push_str(str::from_utf8(&data.to_vec()).unwrap());
        Ok(data.len())
    }
}

const DEFAULT_SERVER : &str = "ece459.patricklam.ca:4590";
impl Drop for Packages {
    fn drop(&mut self) {
        self.execute()
    }
}

static EASYKEY_COUNTER: AtomicI32 = AtomicI32::new(0);

pub struct AsyncState {
    server : String,
    easys: Vec<Easy2Handle<Collector>>,
    multi: Multi,
    easy_num_to_pkg_name: HashMap<i32, String>,
    easy_num_to_version: HashMap<i32, String>,
    easy_num_to_md5sum: HashMap<i32, String>
}

impl AsyncState {
    pub fn new() -> AsyncState {
        AsyncState {
            server : String::from(DEFAULT_SERVER),
            easys: Vec::new(),
            multi: Multi::new(),
            easy_num_to_pkg_name: HashMap::new(),
            easy_num_to_version: HashMap::new(),
            easy_num_to_md5sum: HashMap::new()
        }
    }
}

impl Packages {
    pub fn set_server(&mut self, new_server:&str) {
        self.async_state.server = String::from(new_server);
    }

    /// Retrieves the version number of pkg and calls enq_verify_with_version with that version number.
    pub fn enq_verify(&mut self, pkg:&str) {
        let version = self.get_available_debver(pkg);
        match version {
            None => { println!("Error: package {} not defined.", pkg); return },
            Some(v) => { 
                let vs = &v.to_string();
                self.enq_verify_with_version(pkg, vs); 
            }
        };
    }

    /// Enqueues a request for the provided version/package information. Stores any needed state to async_state so that execute() can handle the results and print out needed output.
    pub fn enq_verify_with_version(&mut self, pkg:&str, version:&str) {
        let url = format!("http://{}/rest/v1/checksums/{}/{}", self.async_state.server, pkg, urlencoding::encode(version));
        println!("queueing request {}", url);

        let mut easy = Easy2::new(Collector(Box::new(String::from(""))));
        easy.url(&*url).unwrap();
        easy.verbose(false).unwrap();
        let handle = self.async_state.multi.add2(easy).unwrap();
        self.async_state.easys.push(handle);

        let easy_num = EASYKEY_COUNTER.load(Ordering::SeqCst);
        self.async_state.easy_num_to_pkg_name.insert(easy_num, String::from(pkg));
        self.async_state.easy_num_to_version.insert(easy_num, String::from(version));
        self.async_state.easy_num_to_md5sum.insert(easy_num, String::from(self.get_md5sum(pkg).unwrap()));
        EASYKEY_COUNTER.fetch_add(1, Ordering::SeqCst);
    }

    /// Asks curl to perform all enqueued requests. For requests that succeed with response code 200, compares received MD5sum with local MD5sum (perhaps stored earlier). For requests that fail with 400+, prints error message.
    pub fn execute(&mut self) {
        self.async_state.multi.pipelining(true, true).unwrap();
        while self.async_state.multi.perform().unwrap() > 0 {
            self.async_state.multi.wait(&mut [], Duration::from_millis(500)).unwrap();
        }
        let mut easy_num = 0;
        for easy_handle in self.async_state.easys.drain(..) {
            let mut easy_after = self.async_state.multi.remove2(easy_handle).unwrap();

            let pkg_name = self.async_state.easy_num_to_pkg_name.get(&easy_num).unwrap();
            let pkg_version = self.async_state.easy_num_to_version.get(&easy_num).unwrap();
            let md5sum = self.async_state.easy_num_to_md5sum.get(&easy_num).unwrap();

            let response_code = easy_after.response_code().unwrap();
            if response_code == 200 {
                let same_md5sum = md5sum == easy_after.get_ref().0.as_str();
                println!("verifying {}, matches: {:?}", pkg_name, same_md5sum);
            } else {
                // Assume failed with 400+ error
                println!("got error {} on request for package {} version {}", response_code, pkg_name, pkg_version);
            }
            easy_num += 1;
        }

        // Clean up
        EASYKEY_COUNTER.store(0, Ordering::Relaxed);
        self.async_state.easy_num_to_pkg_name.clear();
        self.async_state.easy_num_to_version.clear();
        self.async_state.easy_num_to_md5sum.clear();
    }
}
