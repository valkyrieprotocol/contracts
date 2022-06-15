use cosmwasm_std::{Uint128, Decimal, Binary, Response, StdResult, StdError, Addr};
use bigint::{U256, U512};
use std::num::ParseIntError;

pub fn make_response(action: &str) -> Response {
    let mut response = Response::new();

    response = response.add_attribute("action", action);

    response
}

pub fn map_u128(value: Vec<Uint128>) -> Vec<u128> {
    value.iter().map(|v| v.u128()).collect()
}

pub fn map_uint128(value: Vec<u128>) -> Vec<Uint128> {
    value.iter().map(|&v| Uint128::from(v)).collect()
}

pub fn split_uint128(value: Uint128, ratio: &Vec<Uint128>) -> Vec<Uint128> {
    let total_amount = ratio.iter().sum::<Uint128>();

    ratio.iter()
        .map(|v| value.multiply_ratio(*v, total_amount))
        .collect()
}

pub fn split_ratio_uint128(value: Uint128, ratio: &Vec<Decimal>) -> Vec<Uint128> {
    ratio.iter().map(|r| *r * value).collect()
}

pub fn to_ratio_uint128(values: &Vec<Uint128>) -> Vec<Decimal> {
    let total_amount = values.iter().sum::<Uint128>();

    values.iter()
        .map(|v| Decimal::from_ratio(*v, total_amount))
        .collect()
}

pub fn parse_uint128(value: &str) -> Result<Uint128, ParseIntError> {
    let r = value.parse::<u128>().map(|v| Uint128::from(v));
    return if r.is_ok() {
        Ok(r.unwrap())
    } else {
        Err(r.unwrap_err() as ParseIntError)
    }
}

pub fn find_mut_or_push<T, P: Fn(&T) -> bool, N: Fn() -> T, F: Fn(&mut T)>(
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
    vec: &[T],
    predicate: P,
) -> Option<&T> {
    for each in vec {
        if predicate(each) {
            return Some(each)
        }
    }

    None
}

static DECIMAL_FRACTION: Uint128 = Uint128::new(1_000_000_000_000_000_000u128);
pub fn calc_ratio_amount(value: Uint128, ratio: Decimal) -> (Uint128, Uint128) {
    let base = value.multiply_ratio(DECIMAL_FRACTION, DECIMAL_FRACTION * ratio + DECIMAL_FRACTION);

    (value.checked_sub(base).unwrap(), base)
}

pub fn add_query_parameter(url: &str, key: &str, value: &str) -> String {
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

pub fn put_query_parameter(url: &str, key: &str, value: &str) -> String {
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
const TERRA_CONTRACT_ADDRESS_LENGTH: usize = 64;
const COMPRESSED_TERRA_ADDRESS_LENGTH: usize = 32;
const COMPRESSED_TERRA_CONTRACT_ADDRESS_LENGTH: usize = 76;
const BECH32_CHARSET: &str = "qpzry9x8gf2tvdw0s3jn54khce6mua7l";

pub fn is_contract(address: &Addr) -> bool {
    address.to_string().len() > TERRA_ADDRESS_LENGTH
}

pub fn compress_addr(address: &str) -> StdResult<String> {
    if address.len() == TERRA_ADDRESS_LENGTH {
        let mut result = U256::zero();
        for c in address[TERRA_ADDRESS_HRP_LENGTH..].chars() {
            let index = BECH32_CHARSET.find(c)
                .ok_or(StdError::generic_err("Does not contain BECH32_CHARSET"))?;
            result = (result << 5) | U256::from(index);
        }

        let mut bytes = [0u8; 32];
        result.to_big_endian(&mut bytes);
        Ok(Binary::from(&bytes[8..]).to_base64())
    } else {
        let mut result = U512::zero();
        for c in address[TERRA_ADDRESS_HRP_LENGTH..].chars() {
            let index = BECH32_CHARSET.find(c)
                .ok_or(StdError::generic_err("Does not contain BECH32_CHARSET"))?;
            result = (result << 5) | U512::from(index);
        }

        let mut bytes = [0u8; 64];
        result.to_big_endian(&mut bytes);
        Ok(Binary::from(&bytes[8..]).to_base64())
    }

}

pub fn decompress_addr(text: &str) -> StdResult<String> {
    let is_contract = if text.len() == COMPRESSED_TERRA_ADDRESS_LENGTH {
        false
    } else if text.len() == COMPRESSED_TERRA_CONTRACT_ADDRESS_LENGTH {
        true
    } else {
        return Err(StdError::generic_err("Invalid compressed addr."));
    };


    let decoded = Binary::from_base64(text).unwrap();
    let mut result = String::new();
    if !is_contract {
        let mut bytes = [0u8; 32];
        bytes[8..].clone_from_slice(decoded.as_slice());

        let mut data = U256::from_big_endian(&bytes);

        for _ in TERRA_ADDRESS_HRP_LENGTH..TERRA_ADDRESS_LENGTH {
            let index = (data & U256::from(0x1F)).as_u32() as usize;
            result = BECH32_CHARSET.chars().nth(index)
                .ok_or(StdError::generic_err("Does not contain BECH32_CHARSET"))?
                .to_string() + &result;
            data = data >> 5;
        }
    } else {
        let mut bytes = [0u8; 64];
        bytes[8..].clone_from_slice(decoded.as_slice());

        let mut data = U512::from_big_endian(&bytes);

        for _ in TERRA_ADDRESS_HRP_LENGTH..TERRA_CONTRACT_ADDRESS_LENGTH {
            let index = (data & U512::from(0x1F)).as_u32() as usize;
            result = BECH32_CHARSET.chars().nth(index)
                .ok_or(StdError::generic_err("Does not contain BECH32_CHARSET"))?
                .to_string() + &result;
            data = data >> 5;
        }
    }

    Ok(TERRA_ADDRESS_HRP.to_string() + &result)
}

pub fn is_valid_schedule(distribution_schedule: &Vec<(u64, u64, Uint128)>) -> bool {
    let mut check_block = 0;

    for (start, end, _amount) in distribution_schedule.iter() {
        if start.clone() < check_block || start.clone() >= end.clone() {
            return false;
        }
        check_block = end.clone();
    }

    return true;
}

pub fn validate_zero_to_one(value: Decimal, name: &str) -> StdResult<()> {
    if Decimal::zero() <= value && value <= Decimal::one() {
        Ok(())
    } else {
        Err(StdError::generic_err(format!("{} must be 0 to 1", name)))
    }
}
