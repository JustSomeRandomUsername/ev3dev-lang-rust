//! Helper struct that manages attributes.
//! It creates an `Attribute` instance if it does not exists or uses a cached one.

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{self, Debug};
use std::fs;
use std::string::String;

use crate::{utils::OrErr, Attribute, Ev3Error, Ev3Result, Port};

/// The root driver path `/sys/class/`.
const ROOT_PATH: &str = "/sys/class/";

/// Helper struct that manages attributes.
/// It creates an `Attribute` instance if it does not exists or uses a cached one.
pub struct Driver {
    class_name: String,
    name: String,
    attributes: RefCell<HashMap<String, Attribute>>,
}

impl Driver {
    /// Returns a new `Driver`.
    /// All attributes created by this driver will use the path `/sys/class/{class_name}/{name}`.
    pub fn new(class_name: &str, name: &str) -> Driver {
        Driver {
            class_name: class_name.to_owned(),
            name: name.to_owned(),
            attributes: RefCell::new(HashMap::new()),
        }
    }

    /// Returns the name of the device with the given `class_name`, `driver_name` and at the given `port`.
    ///
    /// Returns `Ev3Error::NotFound` if no such device exists.
    pub fn find_name_by_port_and_driver(
        class_name: &str,
        port: &dyn Port,
        driver_name: &str,
    ) -> Ev3Result<String> {
        let port_address = port.address();

        let paths = fs::read_dir(format!("{}{}", ROOT_PATH, class_name))?;

        for path in paths {
            let file_name = path?.file_name();
            let name = file_name.to_str().or_err()?;

            let address = Attribute::new(class_name, name, "address")?;

            if address.get::<String>()?.contains(&port_address) {
                let driver = Attribute::new(class_name, name, "driver_name")?;

                if driver.get::<String>()? == driver_name {
                    return Ok(name.to_owned());
                }
            }
        }

        Err(Ev3Error::NotFound)
    }

    /// Returns the name of the device with the given `class_name` and at the given `port`.
    ///
    /// Returns `Ev3Error::NotFound` if no such device exists.
    /// Returns `Ev3Error::MultipleMatches` if more then one matching device exists.
    pub fn find_name_by_port(class_name: &str, port: &dyn Port) -> Ev3Result<String> {
        let port_address = port.address();

        let paths = fs::read_dir(format!("{}{}", ROOT_PATH, class_name))?;

        for path in paths {
            let file_name = path?.file_name();
            let name = file_name.to_str().or_err()?;

            let address = Attribute::new(class_name, name, "address")?;

            if address.get::<String>()?.contains(&port_address) {
                return Ok(name.to_owned());
            }
        }

        Err(Ev3Error::NotFound)
    }

    /// Returns the name of the device with the given `class_name`.
    ///
    /// Returns `Ev3Error::NotFound` if no such device exists.
    /// Returns `Ev3Error::MultipleMatches` if more then one matching device exists.
    pub fn find_name_by_driver(class_name: &str, driver_name: &str) -> Ev3Result<String> {
        let mut names = Driver::find_names_by_driver(class_name, driver_name)?;

        match names.len() {
            0 => Err(Ev3Error::NotFound),
            1 => Ok(names
                .pop()
                .expect("Name vector contains exactly one element")),
            _ => Err(Ev3Error::MultipleMatches),
        }
    }

    /// Returns the names of the devices with the given `class_name`.
    pub fn find_names_by_driver(class_name: &str, driver_name: &str) -> Ev3Result<Vec<String>> {
        let paths = fs::read_dir(format!("{}{}", ROOT_PATH, class_name))?;

        let mut found_names = Vec::new();
        for path in paths {
            let file_name = path?.file_name();
            let name = file_name.to_str().or_err()?;

            let driver = Attribute::new(class_name, name, "driver_name")?;

            if driver.get::<String>()? == driver_name {
                found_names.push(name.to_owned());
            }
        }

        Ok(found_names)
    }

    /// Return the `Attribute` wrapper for the given `attribute_name`.
    /// Creates a new one if it does not exist.
    pub fn get_attribute(&self, attribute_name: &str) -> Attribute {
        let mut attributes = self.attributes.borrow_mut();

        if !attributes.contains_key(attribute_name) {
            if let Ok(v) =
                Attribute::new(self.class_name.as_ref(), self.name.as_ref(), attribute_name)
            {
                attributes.insert(attribute_name.to_owned(), v);
            };
        };

        attributes
            .get(attribute_name)
            .expect("Internal error in the attribute map")
            .clone()
    }
}

impl Clone for Driver {
    fn clone(&self) -> Self {
        Driver {
            class_name: self.class_name.clone(),
            name: self.name.clone(),
            attributes: RefCell::new(HashMap::new()),
        }
    }
}

impl Debug for Driver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Driver {{ class_name: {}, name: {} }}",
            self.class_name, self.name
        )
    }
}
