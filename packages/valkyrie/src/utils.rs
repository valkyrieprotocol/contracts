use cosmwasm_std::{Uint128, Decimal, StdResult, Binary};
use bigint::U256;

pub fn map_u128(value: Vec<Uint128>) -> Vec<u128> {
    value.iter().map(|v| v.u128()).collect()
}

pub fn map_uint128(value: Vec<u128>) -> Vec<Uint128> {
    value.iter().map(|&v| Uint128::from(v)).collect()
}

pub fn find_mut_or_push<T, P: Fn(&T) -> bool, N: Fn() -> T, F: Fn(&mut T) -> ()>(
    vec: &mut Vec<T>,
    predicate: P,
    new: N,
    f: F,
) {
    let item = vec.iter_mut().find(|v| predicate(*v));

    match item {
        None => vec.push(new()),
        Some(item) => f(item),
    }
}

pub fn find<T, P: Fn(&T) -> bool>(
    vec: &Vec<T>,
    predicate: P,
) -> Option<&T> {
    for each in vec {
        if predicate(each) {
            return Some(each)
        }
    }

    return None
}

static DECIMAL_FRACTION: Uint128 = Uint128(1_000_000_000_000_000_000u128);
pub fn calc_ratio_amount(value: u128, ratio: Decimal) -> (u128, u128) {
    let value = Uint128::from(value);
    let base = value.multiply_ratio(DECIMAL_FRACTION, DECIMAL_FRACTION * ratio + DECIMAL_FRACTION);

    (value.checked_sub(base).unwrap().u128(), base.u128())
}

pub fn add_query_parameter(url: &String, key: &String, value: &String) -> String {
    let mut result = String::from(url);

    if result.contains('?') {
        if !(result.ends_with('&') || result.ends_with('?')) {
            result.push('&');
        }
    } else {
        result.push('?');
    }
    result.push_str(&key);
    result.push('=');
    result.push_str(&value);

    result
}

pub fn put_query_parameter(url: &String, key: &String, value: &String) -> String {
    let query_start_index = url.find('?');
    if query_start_index.is_none() {
        return add_query_parameter(url, key, value);
    }
    let query_start_index = query_start_index.unwrap();

    let query_string = &url[query_start_index..];
    let mut key_start_index = query_string.find(format!("?{}", key).as_str());
    if key_start_index.is_none() {
        key_start_index = query_string.find(format!("&{}", key).as_str());
    }
    if key_start_index.is_none() {
        return add_query_parameter(url, key, value);
    }
    let key_start_index = query_start_index + key_start_index.unwrap() + 1;

    let mut result = String::from(url);

    let key_end_index = key_start_index + key.len() - 1;
    if key_end_index >= url.len() - 1 {
        result.push('=');
    } else if url.chars().nth(key_end_index + 1).unwrap() != '=' {
        result.insert(key_end_index + 1, '=');
    }

    let value_start_index = key_start_index + key.len() + 1;
    if value_start_index > url.len() - 1 {
        result.push_str(value);
        return result
    }

    let mut value_end_index = result[value_start_index..].find('&');
    while value_end_index.is_some() && result[value_end_index.unwrap()..].starts_with("&amp") {
        value_end_index = result[(value_end_index.unwrap() + 1)..].find('&');
    }
    let value_end_index = value_end_index.unwrap_or(url.len() - 1);

    if value_start_index > value_end_index {
        result.insert_str(value_start_index, value);
    } else {
        result.replace_range(value_start_index..(value_end_index + 1), "");
        result.insert_str(value_start_index, value);
    }

    result
}

const TERRA_ADDRESS_HRP: &str = "terra1";
const TERRA_ADDRESS_HRP_LENGTH: usize = 6;
const TERRA_ADDRESS_LENGTH: usize = 44;
const BECH32_CHARSET: &str = "qpzry9x8gf2tvdw0s3jn54khce6mua7l";
pub fn compress_addr(address: &String) -> String {
    let mut result = U256::zero();
    for c in address[TERRA_ADDRESS_HRP_LENGTH..].chars() {
        let index = BECH32_CHARSET.find(c).unwrap();
        result = (result << 5) | U256::from(index);
    }

    let mut bytes = [0u8, 32];
    result.to_big_endian(&mut bytes);

    Binary::from(&bytes).to_base64()
}

pub fn decompress_addr(text: &String) -> String {
    let mut bytes = Binary::from_base64(text).unwrap().as_slice();
    let mut data = U256::from_big_endian(bytes);
    let mut result = String::new();

    for _ in TERRA_ADDRESS_HRP_LENGTH..TERRA_ADDRESS_LENGTH {
        let index = (data & U256::from(0x1F)).as_u32() as usize;
        result = BECH32_CHARSET.chars().nth(index).unwrap().to_string() + &result;
        data = data >> 5;
    }

    TERRA_ADDRESS_HRP + &result
}
