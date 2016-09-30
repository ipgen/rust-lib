extern crate crypto;
extern crate ipnetwork;

use std::net::Ipv6Addr;
use std::str::FromStr;
use crypto::blake2b::Blake2b;
use crypto::digest::Digest;
use ipnetwork::Ipv6Network;

/// Generates an IPv6 address
///
/// `ip6gen` takes any string and a unique IPv6 local address
/// prefix eg `fd52:f6b0:3162::/64` and computes a unique IP address.
pub fn ip(name: &str, cidr: &str) -> Result<Ipv6Addr, String> {
    let net = match Ipv6Network::from_str(cidr).map_err(|err| format!("{:?}", err)) {
        Ok(net) => {
            if net.prefix() == 128 {
                return Err(format!("{}/{} is already a full IPv6 address",
                                   net.ip(),
                                   net.prefix()));
            } else {
                net
            }
        }
        Err(msg) => return Err(msg),
    };
    ip6(name, net)
}

fn ip6(name: &str, net: Ipv6Network) -> Result<Ipv6Addr, String> {
    // If we divide the prefix by 4 we will get the total number
    // of characters that we must never touch.
    let network_len = net.prefix() as usize / 4;
    let ip = net.ip().segments();
    // Uncompress the IP address and throw away the semi-colons
    // so we can easily join extract the network part and later
    // join it to the address part that we will compute.
    let ip_parts: Vec<String> = ip.iter()
        .map(|b| format!("{:04x}", b))
        .collect();
    let ip_hash = ip_parts.join("");
    let ip_hash = ip_hash.as_str();
    let network_hash = &ip_hash[0..network_len];
    // The number of characters we need to generate
    //
    // * An IPv6 address has a total number of 32 (8*4) characters.
    // * Subtracting those characters from the total in an IP address
    //   gives us the number of characters we need to generate.
    let address_len = 32 - network_len;
    // Blake2b generates hashes in multiples of 2 so we need to divide
    // the total number of characters we need by 2. Sadly this means we
    // can't always fully utilise the address space we need to fill.
    let hash_is_bigger = address_len % 2 != 0;
    let mut blake_len = address_len / 2;
    if hash_is_bigger {
        blake_len += 1;
    };
    let hash = hash(name, blake_len);
    let address_hash = if hash_is_bigger {
        &hash[..hash.len()]
    } else {
        hash.as_str()
    };
    let ip_hash = format!("{}{}", network_hash, address_hash);
    let ip = format!("{}:{}:{}:{}:{}:{}:{}:{}",
                     &ip_hash[0..4],
                     &ip_hash[4..8],
                     &ip_hash[8..12],
                     &ip_hash[12..16],
                     &ip_hash[16..20],
                     &ip_hash[20..24],
                     &ip_hash[24..28],
                     &ip_hash[28..32]);
    Ipv6Addr::from_str(ip.as_str())
        .map_err(|err| format!("generated IPv6 address ({}) has {}", ip, err))
}

// Calculate a hash for the subnet
pub fn subnet(name: &str) -> String {
    hash(name, 2)
}

fn hash(name: &str, len: usize) -> String {
    let mut hash = Blake2b::new(len);
    hash.input_str(name);
    hash.result_str()
}

#[cfg(test)]
mod test {
    #[test]
    fn ip_is_valid() {
        match super::ip("c0a010fb-2632-40cb-a105-90297cba567a",
                         "fd52:f6b0:3162::/48") {
            Ok(_) => {
                // yay!
            }
            Err(err) => panic!(err),
        };
    }

}
