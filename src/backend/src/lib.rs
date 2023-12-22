#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::collections::HashMap;
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct WaterUsage {
    id: u64,
    timestamp: u64,
    gallons_used: f64,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct WaterConservationTip {
    id: u64,
    tip_text: String,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct WaterCondition {
    id: u64,
    location: String,
    water_level: f64,
    timestamp: u64,
}

// Implementing Storable and BoundedStorable traits for WaterConservationTip
impl Storable for WaterConservationTip {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for WaterConservationTip {
    const MAX_SIZE: u32 = 1024; // Set an appropriate value based on your data
    const IS_FIXED_SIZE: bool = false;
}

// Implementing Storable and BoundedStorable traits for WaterUsage
impl Storable for WaterUsage {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for WaterUsage {
    const MAX_SIZE: u32 = 1024; // Set an appropriate value based on your data
    const IS_FIXED_SIZE: bool = false;
}

// Implementing Storable and BoundedStorable traits for WaterCondition
impl Storable for WaterCondition {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for WaterCondition {
    const MAX_SIZE: u32 = 1024; // Set an appropriate value based on your data
    const IS_FIXED_SIZE: bool = false;
}

impl Storable for WaterConservationApp {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for WaterConservationApp {
    const MAX_SIZE: u32 = 1024; // Set an appropriate value based on your data
    const IS_FIXED_SIZE: bool = false;
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct WaterConservationApp {
    water_usage_map: HashMap<u64, WaterUsage>,
    conservation_tip_map: HashMap<u64, WaterConservationTip>,
    water_condition_map: HashMap<u64, WaterCondition>,
}

thread_local! {
    static WATER_MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static WATER_ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(WATER_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter for water usage items")
    );

    static WATER_USAGE: RefCell<StableBTreeMap<u64, WaterUsage, Memory>> =
        RefCell::new(StableBTreeMap::init(
            WATER_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));

    static TIP_STORAGE: RefCell<StableBTreeMap<u64, WaterConservationTip, Memory>> = RefCell::new(
        StableBTreeMap::init(
            WATER_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))) // use a different MemoryId
        )
    );
    static CONDITION: RefCell<StableBTreeMap<u64, WaterCondition, Memory>> = RefCell::new(
        StableBTreeMap::init(
            WATER_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))) // use a different MemoryId
        )
    );
}

// // Helper method to perform insert for WaterUsage
// fn do_insert_water_usage(item: &WaterUsage) {
//     WATER_STORAGE.with(|service| {
//         service.borrow_mut().water_usage_map.insert(item.id, item.clone());
//     });
// }
// Helper method to perform insert for WaterUsage
fn do_insert_water_usage(item: &WaterUsage) {
    WATER_USAGE.with(|storage| {
        storage.borrow_mut().insert(item.id, item.clone());
    });
}

fn do_insert_conservation_tip(item: WaterConservationTip) {
    TIP_STORAGE.with(|storage| {
        storage.borrow_mut().insert(item.id, item);
    });
}

// Helper method to perform insert for WaterCondition
fn do_insert_water_condition(item: WaterCondition) {
    CONDITION.with(|service| {
        service.borrow_mut().insert(item.id, item);
    });
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct WaterUsagePayload {
    gallons_used: f64,
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct WaterConservationTipPayload {
    tip_text: String,
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct WaterConditionPayload {
    location: String,
    water_level: f64,
}

#[ic_cdk::query]
fn get_water_usage(id: u64) -> Result<WaterUsage, Error> {
    match _get_water_usage(&id) {
        Some(item) => Ok(item),
        None => Err(Error::NotFound {
            msg: format!("water usage with id={} not found", id),
        }),
    }
}

fn _get_water_usage(id: &u64) -> Option<WaterUsage> {
    WATER_USAGE.with(|s| s.borrow().get(id).clone())
}

#[ic_cdk::update]
fn add_water_usage(item: WaterUsagePayload) -> Option<WaterUsage> {
    let id = WATER_ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter for water usage");
    let water_usage = WaterUsage {
        id,
        timestamp: time(),
        gallons_used: item.gallons_used,
    };
    do_insert_water_usage(&water_usage);
    Some(water_usage)
}

#[ic_cdk::update]
fn update_water_usage(id: u64, item: WaterUsagePayload) -> Result<WaterUsage, Error> {
    match WATER_USAGE.with(|service| service.borrow().get(&id)) {
        Some(mut water_usage) => {
            water_usage.gallons_used = item.gallons_used;
            do_insert_water_usage(&water_usage);
            Ok(water_usage)
        }
        None => Err(Error::NotFound {
            msg: format!("couldn't update water usage with id={}. item not found", id),
        }),
    }
}

#[ic_cdk::update]
fn delete_water_usage(id: u64) -> Result<WaterUsage, Error> {
    match WATER_USAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(water_usage) => Ok(water_usage),
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't delete water usage with id={}. item not found.",
                id
            ),
        }),
    }
}

#[ic_cdk::query]
fn get_conservation_tip(id: u64) -> Result<WaterConservationTip, Error> {
    match _get_conservation_tip(&id) {
        Some(item) => Ok(item),
        None => Err(Error::NotFound {
            msg: format!("conservation tip with id={} not found", id),
        }),
    }
}

fn _get_conservation_tip(id: &u64) -> Option<WaterConservationTip> {
    TIP_STORAGE.with(|s| s.borrow().get(id))
}

#[ic_cdk::update]
fn add_conservation_tip(item: WaterConservationTipPayload) -> Option<WaterConservationTip> {
    let id = WATER_ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter for conservation tips");
    let conservation_tip = WaterConservationTip {
        id,
        tip_text: item.tip_text,
    };
    do_insert_conservation_tip(conservation_tip.clone());
    Some(conservation_tip)
}

#[ic_cdk::update]
fn update_conservation_tip(
    id: u64,
    item: WaterConservationTipPayload,
) -> Result<WaterConservationTip, Error> {
    match TIP_STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut conservation_tip) => {
            conservation_tip.tip_text = item.tip_text;
            do_insert_conservation_tip(conservation_tip.clone());
            Ok(conservation_tip)
        }
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't update conservation tip with id={}. item not found",
                id
            ),
        }),
    }
}

#[ic_cdk::update]
fn delete_conservation_tip(id: u64) -> Result<WaterConservationTip, Error> {
    match TIP_STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(conservation_tip) => Ok(conservation_tip),
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't delete conservation tip with id={}. item not found.",
                id
            ),
        }),
    }
}

#[ic_cdk::query]
fn get_water_condition(id: u64) -> Result<WaterCondition, Error> {
    match _get_water_condition(&id) {
        Some(item) => Ok(item),
        None => Err(Error::NotFound {
            msg: format!("water condition with id={} not found", id),
        }),
    }
}

fn _get_water_condition(id: &u64) -> Option<WaterCondition> {
    CONDITION.with(|s| s.borrow().get(id))
}

#[ic_cdk::update]
fn add_water_condition(item: WaterConditionPayload) -> Option<WaterCondition> {
    let id = WATER_ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter for water conditions");
    let water_condition = WaterCondition {
        id,
        location: item.location,
        water_level: item.water_level,
        timestamp: time(),
    };
    do_insert_water_condition(water_condition.clone());
    Some(water_condition)
}

#[ic_cdk::update]
fn update_water_condition(id: u64, item: WaterConditionPayload) -> Result<(), Error> {
    match CONDITION.with(|service| service.borrow().get(&id)) {
        Some(mut water_condition) => {
            water_condition.location = item.location;
            water_condition.water_level = item.water_level;
            do_insert_water_condition(water_condition);
            Ok(())
        }
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't update water condition with id={}. item not found",
                id
            ),
        }),
    }
}

#[ic_cdk::update]
fn delete_water_condition(id: u64) -> Result<WaterCondition, Error> {
    match CONDITION.with(|service| service.borrow_mut().remove(&id)) {
        Some(water_condition) => Ok(water_condition),
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't delete water condition with id={}. item not found.",
                id
            ),
        }),
    }
}

#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
}

// To generate the Candid interface definitions for our canister
ic_cdk::export_candid!();
