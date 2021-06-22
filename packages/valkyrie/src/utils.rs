use cosmwasm_std::{Uint128, Decimal, StdResult};

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

// const TERRA_ADDRESS_PREFIX: &str = "terra1";
pub fn compress_addr(address: &String) -> StdResult<String> {
    // // '6' is hrp + delimiter length of terra address
    // let data_chars = address[6..].chars();
    // let char_count = address.len() - 6;
    //
    // let mut boxed = bitbox![Lsb0, u8; 0; char_count * 5];
    //
    // unsafe {
    //     for (_, c) in data_chars.enumerate() {
    //         let bech_index: Vec<bool> = bech32_index_of(c).unwrap().to_be_bytes().as_bits()
    //             .iter()
    //             .map(|v: BitRef<Const, Lsb0, u8>| v.into_bitptr().read())
    //             .collect();
    //
    //         boxed.shift_left(5);
    //         boxed |= bech_index;
    //     }
    // }
    //
    // Ok(Binary::from(boxed.as_slice()).to_base64())

    Ok(address.to_string())
}

pub fn decompress_addr(text: &String) -> StdResult<String> {
    // let mut data = Binary::from_base64(text.as_str())?;
    // let bits: &mut BitSlice<Lsb0, u8> = data.as_slice().view_bits_mut();
    //
    // let mut result: Vec<char> = vec![];
    // // '38' is data part length of terra address
    // for _ in 1..38 {
    //     let byte = &bits[..bits.len() - 5];
    //     // byte.bitand_assign(0x1F.to_be_bytes());
    //
    //     let index = usize::from_be_bytes([0, 0, 0, byte.as_raw_slice()[0]]);
    //     result.insert(0, bech32_char_of(index).unwrap());
    //
    //     bits.shift_right(5);
    // }
    //
    // Ok(result.iter().collect())

    Ok(text.to_string())
}

// static BECH32: &str = "qpzry9x8gf2tvdw0s3jn54khce6mua7l";
// fn bech32_index_of(value: char) -> Option<usize> {
//     let mut i = 0;
//
//     for c in BECH32.chars() {
//         if c == value {
//             return Some(i)
//         }
//
//         i += 1;
//     }
//
//     None
// }
//
// fn bech32_char_of(index: usize) -> Option<char> {
//     let chars: Vec<char> = BECH32.chars().collect();
//
//     if index < 32 {
//         Some(chars[index])
//     } else {
//         None
//     }
// }