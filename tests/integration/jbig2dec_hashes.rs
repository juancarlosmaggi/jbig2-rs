use crate::common::load_test_file;
use jbig2_rs::Jbig2Document;
use sha1::{Digest, Sha1};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Clone, Copy)]
struct KnownFileHash {
    filename: &'static str,
    file_hash: &'static str,
    decode_hash: &'static str,
}

fn test_timeout() -> Duration {
    let seconds = std::env::var("JBIG2_RS_TEST_TIMEOUT_SECS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(30);
    Duration::from_secs(seconds)
}

fn sha1_hex(data: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    let digest = hasher.finalize();
    let mut out = String::with_capacity(40);
    for byte in digest {
        out.push_str(&format!("{:02x}", byte));
    }
    out
}

fn decode_document_hash(document: &Jbig2Document) -> String {
    let mut hasher = Sha1::new();
    for page in &document.pages {
        hasher.update(&page.bit_packed_data);
    }
    let digest = hasher.finalize();
    let mut out = String::with_capacity(40);
    for byte in digest {
        out.push_str(&format!("{:02x}", byte));
    }
    out
}

fn run_case(case: &KnownFileHash) -> Result<(), String> {
    let data = load_test_file(case.filename);
    let input_hash = sha1_hex(&data);
    if input_hash != case.file_hash {
        return Err(format!(
            "fixture hash mismatch for {}: expected {}, got {}",
            case.filename, case.file_hash, input_hash
        ));
    }

    let document = Jbig2Document::parse(&data)
        .map_err(|err| format!("Failed to parse JBIG2 {}: {}", case.filename, err))?;
    let decoded_hash = decode_document_hash(&document);
    if decoded_hash != case.decode_hash {
        return Err(format!(
            "decoded hash mismatch for {}: expected {}, got {}",
            case.filename, case.decode_hash, decoded_hash
        ));
    }
    Ok(())
}

fn run_case_with_timeout(case: KnownFileHash) {
    let timeout = test_timeout();
    let (sender, receiver) = mpsc::channel();

    thread::spawn(move || {
        let result = std::panic::catch_unwind(|| run_case(&case))
            .map_err(|panic| {
                if let Some(message) = panic.downcast_ref::<&str>() {
                    message.to_string()
                } else if let Some(message) = panic.downcast_ref::<String>() {
                    message.clone()
                } else {
                    "panic in hash test".to_string()
                }
            })
            .and_then(|result| result);

        let _ = sender.send(result);
    });

    match receiver.recv_timeout(timeout) {
        Ok(Ok(())) => {}
        Ok(Err(message)) => panic!("{}", message),
        Err(mpsc::RecvTimeoutError::Timeout) => {
            panic!(
                "hash test timed out after {}s for {}",
                timeout.as_secs(),
                case.filename
            );
        }
        Err(err) => panic!("hash test channel error for {}: {}", case.filename, err),
    }
}

macro_rules! hash_case {
    ($name:ident, $filename:literal, $file_hash:literal, $decode_hash:literal) => {
        #[test]
        fn $name() {
            run_case_with_timeout(KnownFileHash {
                filename: $filename,
                file_hash: $file_hash,
                decode_hash: $decode_hash,
            });
        }
    };
}

hash_case!(
    ubc_042_1,
    "ubc/042_1.jb2",
    "673e1ee5c55ab241b171e476ba1168a42733ddaa",
    "ebfdf6e2fc5ff3ee2271c2fa19de0e52712046e8"
);
hash_case!(
    ubc_042_2,
    "ubc/042_2.jb2",
    "9aa2804e2d220952035c16fb3c907547884067c5",
    "ebfdf6e2fc5ff3ee2271c2fa19de0e52712046e8"
);
hash_case!(
    ubc_042_3,
    "ubc/042_3.jb2",
    "9663a5f35727f13e61a0a2f0a64207b1f79e7d67",
    "ebfdf6e2fc5ff3ee2271c2fa19de0e52712046e8"
);
hash_case!(
    ubc_042_4,
    "ubc/042_4.jb2",
    "014df658c8b99b600c2ceac3f1d53c7cc2b4917c",
    "ebfdf6e2fc5ff3ee2271c2fa19de0e52712046e8"
);
hash_case!(
    ubc_042_5,
    "ubc/042_5.jb2",
    "264720a6ccbbf72aa6a2cfb6343f43b8e6f2da4b",
    "ebfdf6e2fc5ff3ee2271c2fa19de0e52712046e8"
);
hash_case!(
    ubc_042_6,
    "ubc/042_6.jb2",
    "96f7dc9df4a1b305f9ac082dd136f85ef5b108fe",
    "ebfdf6e2fc5ff3ee2271c2fa19de0e52712046e8"
);
hash_case!(
    ubc_042_7,
    "ubc/042_7.jb2",
    "5526371ba9dc2b8743f20ae3e05a7e60b3dcba76",
    "ebfdf6e2fc5ff3ee2271c2fa19de0e52712046e8"
);
hash_case!(
    ubc_042_8,
    "ubc/042_8.jb2",
    "4bf0c87dfaf40d67c36f2a083579eeda26d54641",
    "ebfdf6e2fc5ff3ee2271c2fa19de0e52712046e8"
);
hash_case!(
    ubc_042_9,
    "ubc/042_9.jb2",
    "53e630e7fe2fe6e1d6164758e15fc93382e07f55",
    "ebfdf6e2fc5ff3ee2271c2fa19de0e52712046e8"
);
hash_case!(
    ubc_042_10,
    "ubc/042_10.jb2",
    "5ca1364367e25cb8f642e9dc677a94d5cfed0c8b",
    "ebfdf6e2fc5ff3ee2271c2fa19de0e52712046e8"
);
hash_case!(
    ubc_042_11,
    "ubc/042_11.jb2",
    "bc194caf022bc5345fc41259e05cea3c08245216",
    "ebfdf6e2fc5ff3ee2271c2fa19de0e52712046e8"
);
hash_case!(
    ubc_042_12,
    "ubc/042_12.jb2",
    "f354df8eb4849bc707f088739e322d1fe3a14ef3",
    "ebfdf6e2fc5ff3ee2271c2fa19de0e52712046e8"
);
hash_case!(
    ubc_042_13,
    "ubc/042_13.jb2",
    "7d428bd542f58591b254d9827f554b0552c950a7",
    "28a6bd83a8a3a36910fbc1f5ce06c962e4332911"
);
hash_case!(
    ubc_042_14,
    "ubc/042_14.jb2",
    "c40fe3a02acb6359baf9b40fc9c49bc0800be589",
    "28a6bd83a8a3a36910fbc1f5ce06c962e4332911"
);
hash_case!(
    ubc_042_15,
    "ubc/042_15.jb2",
    "a9e39fc1ecb178aec9f05039514d75ea3246246c",
    "ebfdf6e2fc5ff3ee2271c2fa19de0e52712046e8"
);
hash_case!(
    ubc_042_16,
    "ubc/042_16.jb2",
    "4008bbca43670f3c90eaee26516293ba95baaf3d",
    "ebfdf6e2fc5ff3ee2271c2fa19de0e52712046e8"
);
hash_case!(
    ubc_042_17,
    "ubc/042_17.jb2",
    "0ff95637b64c57d659a41c582da03e25321551fb",
    "ebfdf6e2fc5ff3ee2271c2fa19de0e52712046e8"
);
hash_case!(
    ubc_042_18,
    "ubc/042_18.jb2",
    "87381d044f00c4329200e44decbe91bebfa31595",
    "ebfdf6e2fc5ff3ee2271c2fa19de0e52712046e8"
);
hash_case!(
    ubc_042_19,
    "ubc/042_19.jb2",
    "387d95a140b456d4742622c788cf5b51cebbf438",
    "ebfdf6e2fc5ff3ee2271c2fa19de0e52712046e8"
);
hash_case!(
    ubc_042_20,
    "ubc/042_20.jb2",
    "85c19e9ec42b8ddd6b860a1bebea1c67610e7a59",
    "ebfdf6e2fc5ff3ee2271c2fa19de0e52712046e8"
);
hash_case!(
    ubc_042_21,
    "ubc/042_21.jb2",
    "ab535c7d7a61a7b9dc53d546e7419ca78ac7f447",
    "ebfdf6e2fc5ff3ee2271c2fa19de0e52712046e8"
);
hash_case!(
    ubc_042_22,
    "ubc/042_22.jb2",
    "a9e2b365be63716dbde74b0661c3c6efd2a6844d",
    "ebfdf6e2fc5ff3ee2271c2fa19de0e52712046e8"
);
hash_case!(
    ubc_042_23,
    "ubc/042_23.jb2",
    "8ffa40a05e93e10982b38a2233a8da58c1b5c343",
    "ebfdf6e2fc5ff3ee2271c2fa19de0e52712046e8"
);
hash_case!(
    ubc_042_24,
    "ubc/042_24.jb2",
    "2553fe65111c58f6412de51d8cdc71651e778ccf",
    "ebfdf6e2fc5ff3ee2271c2fa19de0e52712046e8"
);
hash_case!(
    ubc_042_25,
    "ubc/042_25.jb2",
    "52de4a3b86252d896a8d783ba71dd0699333dd69",
    "ebfdf6e2fc5ff3ee2271c2fa19de0e52712046e8"
);
hash_case!(
    ubc_amb_1,
    "ubc/amb_1.jb2",
    "d6d6d1c981dc37a09108c1e3ed990aa5b345fa6a",
    "3d4b7992d506894662b53415bd3d0d2a2f8b7953"
);
hash_case!(
    ubc_amb_2,
    "ubc/amb_2.jb2",
    "9af6616a89eb03f8934de72626e301a716366c3c",
    "3d4b7992d506894662b53415bd3d0d2a2f8b7953"
);
hash_case!(
    ubc_200_10_0,
    "ubc/200-10-0.jb2",
    "f6014b43775640ef0874497e0873f8deb291cc32",
    "49cddf903d3451ba23297a6b68502504093979cf"
);
hash_case!(
    ubc_200_10_0_stripe,
    "ubc/200-10-0-stripe.jb2",
    "d19f58cd180afd1ae2afd11c96471e98c7c6f125",
    "ac89ae2046c4859348418830287982b6d60bf39b"
);
hash_case!(
    ubc_200_10_45,
    "ubc/200-10-45.jb2",
    "504297b028810f812cbf075597f589a9fb82121b",
    "38aa99e40c6a746391c26c953223bcd4549cadd0"
);
hash_case!(
    ubc_200_10_45_stripe,
    "ubc/200-10-45-stripe.jb2",
    "0d9f2a63c9fd224a6b60a9b7c0cd658f47551edd",
    "2921889fc5ffaafb348084761aa7c54831ec57ba"
);
hash_case!(
    ubc_200_20_0,
    "ubc/200-20-0.jb2",
    "a40aaf33dd4c3225728ddfc0fad12167ceff1b17",
    "cc1732742d5d68c6d5c3f4eec9d5887e9ee24cd0"
);
hash_case!(
    ubc_200_20_0_stripe,
    "ubc/200-20-0-stripe.jb2",
    "d499a89baf69a1b5f6fa450ec20b21136052b4cd",
    "743aa86e7abc9e238e23d02fbc993b048589282a"
);
hash_case!(
    ubc_200_20_45,
    "ubc/200-20-45.jb2",
    "a39f1e2670f1c08dbd07d14a99965bf7253e6318",
    "7213fb351f65397c12accf662787aa3bc028c40f"
);
hash_case!(
    ubc_200_20_45_stripe,
    "ubc/200-20-45-stripe.jb2",
    "3aa44cdef38fc8e34376480408ca99364ccbf0ee",
    "9021716b3eca4da549508db691655eddc4d51548"
);
hash_case!(
    ubc_200_2_0,
    "ubc/200-2-0.jb2",
    "087f529ba6e3cc5fca3773c1d07e39fb642f5052",
    "534fceffada398444ce065088a37b6d6517a3406"
);
hash_case!(
    ubc_200_2_0_stripe,
    "ubc/200-2-0-stripe.jb2",
    "dc227f7531ccecda08511bda9359864c66a8d230",
    "56f0e25ae5863a75d69a1825b820ba004e48d2c4"
);
hash_case!(
    ubc_200_3_0,
    "ubc/200-3-0.jb2",
    "024a20b82e794eb469b4fae2b4f930c5c079fd6b",
    "57fe3645b028e6c7a68dcf707674f889038ee4b5"
);
hash_case!(
    ubc_200_3_0_stripe,
    "ubc/200-3-0-stripe.jb2",
    "2322db7dc956863b7257d28a212431e304661998",
    "32ea498b28a46bf04e0b799c70114ab99ce7d15e"
);
hash_case!(
    ubc_200_3_45,
    "ubc/200-3-45.jb2",
    "21ba06f8cfcc31b5bd7fa39ad98093180d3e05aa",
    "83e01d0a83d167fe00f7389e5fec0a660841aeef"
);
hash_case!(
    ubc_200_3_45_stripe,
    "ubc/200-3-45-stripe.jb2",
    "6dfe3cbb019ef0c30ecae7d2196b1b3fd7634288",
    "20c2ade5766eeb3a70dca9963029c0a74171064b"
);
hash_case!(
    ubc_200_4_0,
    "ubc/200-4-0.jb2",
    "c7d8d8b8a97388b0fcc6e5e3d8708fbce0881edf",
    "b85fe470db7542789b0632dc87dbdc721e07ddf5"
);
hash_case!(
    ubc_200_4_0_stripe,
    "ubc/200-4-0-stripe.jb2",
    "840f076fd542b2ae8d0d1663ed7efd5683326bc7",
    "0acd5a6f24637dad4b948fa24563b1fae04996be"
);
hash_case!(
    ubc_200_4_45,
    "ubc/200-4-45.jb2",
    "6ed49af06268d57137436ffeea2def6f93ea17eb",
    "5177abf7e9d641ca4f553bd4847134e51bb1159a"
);
hash_case!(
    ubc_200_4_45_stripe,
    "ubc/200-4-45-stripe.jb2",
    "0dfc5b59a046ab05364298b1767334298fa03eeb",
    "944a399d8763007ae0477f69b80ec28d7fbe6edd"
);
hash_case!(
    ubc_200_5_0,
    "ubc/200-5-0.jb2",
    "47770e4144b022790af00098ac830ac8665f62a0",
    "515eaf8e4537bbda841abf3b7ffbd1b4728c7597"
);
hash_case!(
    ubc_200_5_0_stripe,
    "ubc/200-5-0-stripe.jb2",
    "23f784c297c204bc1bf7cd1559a7c38a95097266",
    "c69e97f9e1a7e45d6eb3975ecb8a4a7dd7f09e2e"
);
hash_case!(
    ubc_200_5_45,
    "ubc/200-5-45.jb2",
    "193376e966e8bc22868e38791289e810953b5483",
    "77fff5286023b77316221d5c36a6d40f8b905ca9"
);
hash_case!(
    ubc_200_5_45_stripe,
    "ubc/200-5-45-stripe.jb2",
    "d211863df684b5c113c2e29aec72c6a533681356",
    "efd7b9ae877bf3d71c0baa604a1014e1218ada90"
);
hash_case!(
    ubc_200_6_0,
    "ubc/200-6-0.jb2",
    "e66a8cff6c00575018253a06f9309192cc796fb2",
    "7b5dae69e6f8953463dd29707f77225cd8a543ad"
);
hash_case!(
    ubc_200_6_0_stripe,
    "ubc/200-6-0-stripe.jb2",
    "55ba1b94e73d96defbb7abbe35ccf13b4e1ac89f",
    "e8a1b55780dde4102f37ca5fafeff29bbd30e867"
);
hash_case!(
    ubc_200_6_45,
    "ubc/200-6-45.jb2",
    "71d167f8af4e6c2a3202c26873aedf490e8da8f2",
    "9093ba8bfc65b87dddc310d437b8ca626ee2283c"
);
hash_case!(
    ubc_200_6_45_stripe,
    "ubc/200-6-45-stripe.jb2",
    "abcf8f71f9ce0cb65c43942ecf0cfda7ece5d7ff",
    "397a48e4f3a3261928b2175699104117e36349e6"
);
hash_case!(
    ubc_200_8_0,
    "ubc/200-8-0.jb2",
    "e7004846acb5529d5335c16315d11c188edea89d",
    "8cfa43f514911d35d9666e52ae51bbd93a9bddfc"
);
hash_case!(
    ubc_200_8_0_stripe,
    "ubc/200-8-0-stripe.jb2",
    "0d96be49231e7e5a52c41bfa7303768465a9fa81",
    "021fbcfa12122999cded6beea3b7aa3c7018acbd"
);
hash_case!(
    ubc_200_8_45,
    "ubc/200-8-45.jb2",
    "e28403c3bf1014a5b5e9c3c3e5e99cae47aa09ab",
    "669986963011b174d5352d38e6c77f459ff3bebd"
);
hash_case!(
    ubc_200_8_45_stripe,
    "ubc/200-8-45-stripe.jb2",
    "c2e19b3e51d06c102a06643f3ea15f77d6df3788",
    "ceb0ef29cb68fe53d9abceb45ab182c1e6a39ff7"
);
hash_case!(
    ubc_200_lossless,
    "ubc/200-lossless.jb2",
    "b9989aea1a3edd65e38e7fbeaa89a29d7a2aa342",
    "94d9324437bc27955e610ef4fbbd684ad3107fea"
);
hash_case!(
    ubc_600_10_0,
    "ubc/600-10-0.jb2",
    "46c9af206382243d838f86ea45c63e7ca2900b68",
    "0ad323815315270f02f8220ed3b69133a1639f74"
);
hash_case!(
    ubc_600_10_45,
    "ubc/600-10-45.jb2",
    "1f143e95bf57d8d2696525797e198efd785f7221",
    "16002bb4e4cefbb58da5dde531b1064b9e6ad1a7"
);
hash_case!(
    ubc_600_20_0,
    "ubc/600-20-0.jb2",
    "8c874b1fb89e714ef8c64f33d292db2aea4fd05f",
    "a537aac28d9e0ea27d43a38024962f86aa1e403b"
);
hash_case!(
    ubc_600_20_45,
    "ubc/600-20-45.jb2",
    "a9c94915dd140916bc14db7b4bc9fc5d7e73b5a9",
    "5af6ec6f2e8ae68cfb6df3f82bf47ee2f6c4f0b5"
);
hash_case!(
    ubc_600_30_0,
    "ubc/600-30-0.jb2",
    "f0b9eea13b5c7a18742238778f1a3b7e1a4d3361",
    "6feaffc771381922a578bc54c4b50d18e7933ea1"
);
hash_case!(
    ubc_600_30_45,
    "ubc/600-30-45.jb2",
    "65bb4202b575bba6063ef3597a5eefa356b5e660",
    "768788c5176d5ffb5d8d0855d8ab34312611f67d"
);
hash_case!(
    ubc_600_6_0,
    "ubc/600-6-0.jb2",
    "c54abd4bdbb26b1f1209dc03ab10c05cdfd7a63a",
    "baba4bc5359c0fafc54efcba14da2bd5943222be"
);
hash_case!(
    ubc_600_6_45,
    "ubc/600-6-45.jb2",
    "94f4f6ea60eda33e0cd8bb94a5a0f90dc05f96a7",
    "bc3afe7c37533ca43f3244e6877ce38b3e978e9f"
);
hash_case!(
    ubc_600_lossless,
    "ubc/600-lossless.jb2",
    "60ecd5ddfb0984e3d2691bc385f425a50c753019",
    "f632d82b3c3d500098ad560e5ab91c69bd20827f"
);
