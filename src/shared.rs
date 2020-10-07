use md5;

struct Sortvol {
    score: Vec<u8>,
    volume: String,
}

pub fn key2path(key: &Vec<u8>) -> String {
    let mkey = md5::compute(key);
    let b64key = base64::encode(mkey.0);
    format!("/{}/{}/{}", mkey[0], mkey[1], b64key)
}

pub fn key2volumes(
    key: &Vec<u8>,
    volumes: &Vec<String>,
    count: usize,
    svcount: usize,
) -> Vec<String> {
    // this is an intelligent way to pick the volume server for a file
    // stable in the volume server name (not position!)
    // and if more are added the correct portion will move (yay md5!)

    let mut sortvols = vec![];
    for volume in volumes {
        let mut k = key.clone();
        k.append(&mut Vec::from(volume.as_bytes()));
        let digest = md5::compute(k);
        sortvols.push(Sortvol {
            score: digest.to_vec(),
            volume: volume.to_string(),
        });
    }
    sortvols.sort_by_key(|sv| sv.score.clone());

    let ret = sortvols[0..count]
        .iter()
        .map(|sv| {
            if svcount == 1 {
                return sv.volume.clone();
            }
            let svhash = ((sv.score[12] as usize) << 24)
                + ((sv.score[13] as usize) << 16)
                + ((sv.score[14] as usize) << 8)
                + (sv.score[15] as usize);
            let volume = format!("{}/sv{}", sv.volume, svhash & svcount);
            volume
        })
        .collect();
    ret
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_words() {
        let volumes = key2volumes(
            &Vec::from(
                "faosgjaposjgpoajgpoajpogjapojgpoasjgposaasjfosajfposajgpoasjpogjaspogjaop"
                    .as_bytes(),
            ),
            &vec![
                String::from("localhost:3001"),
                String::from("localhost:3002"),
                String::from("localhost:3003"),
                String::from("localhost:3004"),
                String::from("localhost:3005"),
            ],
            4,
            4,
        );
        println!("Volumes: {:?}", volumes);
    }
}
