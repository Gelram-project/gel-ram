// GEL RAM — GEL RAM Noncommercial Reciprocal License 1.0; see LICENSE and NOTICE.
// Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)
#![forbid(unsafe_code)]
mod answer_bench;
mod audit_io;
mod bench_compare;
mod ci_evidence;
mod claims;
mod commitment;
mod crash_series;
mod disclosure;
mod history_crash_series;
mod history_mutation_matrix;
mod host_report;
mod isolation;
mod license_metadata;
mod license_self;
mod measured_sources;
mod memory;
mod mutation_campaign;
mod mutation_matrix;
mod package;
mod process_sequence;
mod property_map;
#[cfg(test)]
mod publication_status_tests;
mod rebuild_check;
mod recorder_lint;
mod reproduce;
mod reproduction;
mod roadmap;
mod source_bundle;
mod source_headers;
mod speed_ab;
mod speed_campaign;
mod speed_convention;
mod tree_scan;
mod ucd_tables;
mod witness;
mod workflow_policy;

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

// Exact reviewed media and source archive only; no general binary exception.
// Pins detect changed bytes; they do not prove decoding safety or semantic truth.
const REVIEWED_ASSETS: &[(&str, &str)] = &[
    (
        "media/gifs/01-evidence-light.gif",
        "53636b0287f01a2651191d954d974e384c91c686b5e3d4bb55bbe3102da18f57",
    ),
    (
        "media/gifs/01-evidence-light.png",
        "4a6e692d1e8a0b86788d2a0e3ee9657e6e6440224f64e3dfbed09c22c105cfe3",
    ),
    (
        "media/gifs/01-evidence-dark.gif",
        "0bb3403ce698f2e5ce6fcc70c04c25a276265e6781b56627b6ca33bb8a8e9d1d",
    ),
    (
        "media/gifs/01-evidence-dark.png",
        "dba278f6e2f0a81d4704e7bf652e22df349a9f5e88798db8d886afbe0ded2883",
    ),
    (
        "media/gifs/02-stale-light.gif",
        "ca2606ca4e62cc298841f07bc0edcb1c11177ec4ed819a5bc5284fe5533a634c",
    ),
    (
        "media/gifs/02-stale-light.png",
        "ea2a12aedc96dd0abd88b5d73817c98bcfbabfa68dcedd6132bb71d9b571a18d",
    ),
    (
        "media/gifs/02-stale-dark.gif",
        "389e6f5f1b9f320a4d85a96122344e77fbcace48fdc0794273738d779d08f7b2",
    ),
    (
        "media/gifs/02-stale-dark.png",
        "a117038d615597d4cba2b54fadbb6dad3ef44abe01df515e59df2a89b93f1d0a",
    ),
    (
        "media/gifs/03-backup-light.gif",
        "82f46988e1b2863180768f11e77fcaf911cb89eeebd63e1e84e1d792932e27fa",
    ),
    (
        "media/gifs/03-backup-light.png",
        "46a309b109085bb02487924bfcffde13df933f619dab1a0e5fdcd0144bbe1e1f",
    ),
    (
        "media/gifs/03-backup-dark.gif",
        "2184ebaaf47fdf5cb086dc2267d6d91c78a7d30ae35c6322f73ad73f2776b610",
    ),
    (
        "media/gifs/03-backup-dark.png",
        "d3e9f5ca90cc52fc0575bdab8d380f6c77597f1d39907db2c6694fd9242286bf",
    ),
    (
        "media/gifs/04-reproduce-light.gif",
        "1adb7e15937b10372e0c19160972cf29581ac17f819d442609457b48d49ba77f",
    ),
    (
        "media/gifs/04-reproduce-light.png",
        "c2bff0a465523f468914f0a04d4f1291e0070d20da7919787e273561a9e4fddc",
    ),
    (
        "media/gifs/04-reproduce-dark.gif",
        "dcaa8afae19d72af698270f4649fd98dd1eb1a3a3bb9271d7ab8c9fb07ac7e2e",
    ),
    (
        "media/gifs/04-reproduce-dark.png",
        "cea27d1bcb3757430b7685bec6611b923247b63ecfa77ba1c75b9d8e4dad7f33",
    ),
    (
        "media/gifs/05-integrity-light.gif",
        "ba6e1e8449c45bffc80923e200fdc502b36616d6c9a8b126be2b79008e7df496",
    ),
    (
        "media/gifs/05-integrity-light.png",
        "c49159370d28a1b5b7f1d6b61b369bc53814de733641f40f83a9671c6533300e",
    ),
    (
        "media/gifs/05-integrity-dark.gif",
        "da4b7699891e9ad334ed7f28155415d305dd08a56620f3b2506dc9187db86719",
    ),
    (
        "media/gifs/05-integrity-dark.png",
        "8ded780ada24b9beb953f6579f69754c92afa4a0e2dc4153edb5b4152621e277",
    ),
    (
        "media/gifs/07-literal-light.gif",
        "603423555fe9ab91d721bc0667c07af7830ff7fc5fbf7f8a7ebea372fc418c33",
    ),
    (
        "media/gifs/07-literal-light.png",
        "4d494435ff510eb122448d5f0c8541fa97ba9b5c12f608e6b04e8e198cd9562f",
    ),
    (
        "media/gifs/07-literal-dark.gif",
        "2659273a529620a822a47a17cbeb1d73bed30920c81dc9eb893e7891b8eb45fd",
    ),
    (
        "media/gifs/07-literal-dark.png",
        "5829514c64f0f50367a8f1dab0ee06d0e76e369fe916789171f88806e2ca4537",
    ),
    // Exact script-free documentation presentation assets; no HTML/SVG wildcard.
    (
        "README-MULTIMEDIA.html",
        "732f35349eec8bb241bafe9bcdad69b37cc5d4c0aa3c45b5993981551b3b311b",
    ),
    (
        "media/presentation/header-light.svg",
        "b54fcff8c5fb76b474b5ac0808fcff85a1db02a83ef5cc7788bf3b4bd9cec5c8",
    ),
    (
        "media/presentation/header-dark.svg",
        "c0755f16a769d392955fdc6b10fafaee9839579a283ea6614c3f559d3197327a",
    ),
    (
        "media/presentation/header-still-light.svg",
        "b2baa837b0108bf86ca62be91b1f7dbcabe6521e1d159983964adcb52c254557",
    ),
    (
        "media/presentation/header-still-dark.svg",
        "933da2774f1fa1bebf1fef64d7662461d9c1388890d165d67151b59b3717de07",
    ),
    (
        "media/presentation/record-light.svg",
        "70acfe3d34f42a7ebfa2ff3da1878d0fc71e0a4d60eee00b747725a852f8572c",
    ),
    (
        "media/presentation/record-dark.svg",
        "ab125b943b68bc8425b5609eaf6bed8cdf4023fcc81b5ae8da15de11a81d6e65",
    ),
    (
        "media/presentation/crash-light.svg",
        "743f09d458407c6c6da9d9f22b862981337c36fae03ad3cacf710ba0e5a89cbc",
    ),
    (
        "media/presentation/crash-dark.svg",
        "ec6c66ec2df71ff2f43d305c6cd584a819881119c41f1c01ab1fcb7e57f68c73",
    ),
    (
        "media/presentation/speed-light.svg",
        "33617e210eee2b077052527ab7d11ba786856a86b7bce29469d92df49915582f",
    ),
    (
        "media/presentation/speed-dark.svg",
        "69ae8d91a9881b27b08bcc6bb9a2a64d8f866be43749f07e14017c2b194a87b2",
    ),
    (
        "media/presentation/core-dark.svg",
        "b88d2c7e1887e4c4a814c012ccd67fcdab1b2df870281ab08d601fae73da471c",
    ),
    (
        "media/presentation/core-light.svg",
        "7bb51b6b5ea707765dff79e6f6842e7194166a9157e8bb09653cdb640f3dbaec",
    ),
    (
        "media/presentation/glance-dark.svg",
        "e495245235c06d52b686ade22c83bac2025f989d5a3bbc39d236d1883892f338",
    ),
    (
        "media/presentation/glance-light.svg",
        "08694c7de9d549e14e11c04855d56683395852d28268447e68e3c36e5ca03305",
    ),
    (
        "media/presentation/pledges-dark.svg",
        "8fd6da72abbb25012c100c4d0eb74fc7bdc56a2fd19711c8873e7e6b15cc2aab",
    ),
    (
        "media/presentation/pledges-light.svg",
        "99000e3ace53834b6fb0283b1c7a29339e26a19b3f40771e6c7fa5ba4fe6e7ce",
    ),
    (
        "media/presentation/rerun-dark.svg",
        "48f48cf6f5ab54d870f5bd262fabc0a2b90a25350d0cc5323099ed777f999c6b",
    ),
    (
        "media/presentation/rerun-light.svg",
        "66d87aed91efa463ec49ea4fcfac6c0d4540732551367f91e3c3490762f10993",
    ),
    (
        "media/presentation/status-dark.svg",
        "dc1313d5741ed331feb75724059703989e1b20fa987768030c31a586301d581b",
    ),
    (
        "media/presentation/status-light.svg",
        "75df85035e87bdda43cf243644ea7c76cbf4da3f7e55c3033a0d7a018bd6da70",
    ),
    (
        "media/presentation/terminal-dark.svg",
        "420dc93197ba66ab22e1367a906cde442ca082042185f2ee86a7d33f77f0fa49",
    ),
    (
        "media/presentation/terminal-light.svg",
        "1728065734553e3a102fa5bb12e83bc917bce1832f3ad6a87eae4c15a38f6a93",
    ),
    (
        "media/presentation/license-light.svg",
        "5dc7211d988bc6db1122b25715786f68f71bde30af17c254015112866de7cb06",
    ),
    (
        "media/presentation/license-dark.svg",
        "b7c516e01a7bae9d99ea68be6587f55da8492039925564cddf96213eda48dbf6",
    ),
    (
        "media/presentation/flow-light.svg",
        "a7e0ad9b7a482712f7aede142e23162580ca1d85b50ac5ce490faadd60bb438d",
    ),
    (
        "media/presentation/facts-light.svg",
        "5dc0ece7ef69cd8950686a4941e2c27b13bc9faa131ab2b24d58833b223958fa",
    ),
    (
        "media/presentation/wall-light.svg",
        "d68cfb645e2e03099e6c345261250ff51b7b2199226d67b81240cc0f683fb9c9",
    ),
    (
        "media/presentation/bars-light.svg",
        "d83fb2f2e9abaa13e7371fd9d6df1a2f7bc50bc92f6bda61a43404b6121e05e0",
    ),
    (
        "media/presentation/logo-light.svg",
        "0bfa881db2ea8d988c385664eb840f8faef1c5e931e37a7e8d6d3f101b67b0b5",
    ),
    (
        "media/presentation/logo-still-light.svg",
        "2cb1b673a1c1791b5cdd30b7194f6b2bf4f3f9c7d909c939470e2de7c79f26e4",
    ),
    (
        "media/presentation/flow-dark.svg",
        "9edc076af21a8b82f31c8e355b284e0b34e3f05707ce81ceaa72f5a822adcb09",
    ),
    (
        "media/presentation/facts-dark.svg",
        "7cf40c42f11a5859c41094223073eb8b7c531599d45171de840d06e835abf45e",
    ),
    (
        "media/presentation/wall-dark.svg",
        "4e1cb7ce5c971ec2c5a90c916b5375378a787caae3e5304457df5b66fccd26b1",
    ),
    (
        "media/presentation/bars-dark.svg",
        "ff0e0e44a62c8d17f7ba8017fafcca308b9510738085194d11166039fa36e5f4",
    ),
    (
        "media/presentation/logo-dark.svg",
        "1bb4a8be9ff7c951a62e1a18e1ee7ba6d9861213df7cd7ca4e1f64413286be6b",
    ),
    (
        "media/presentation/logo-still-dark.svg",
        "0863d48ebe5fbb4199e9947f29518f6f3224250142cb4d4eb4c3c9f68926a132",
    ),
    (
        "media/presentation/chips/01-light.svg",
        "d4207f2334a255e66fc3a92cfeba06ff3ee66dd50b045496c41005c4694f3fc2",
    ),
    (
        "media/presentation/chips/01-dark.svg",
        "68bbe4c1bed153874c101d9e5d751637f2bcbf4068c8778dfbd2e041ed303b1f",
    ),
    (
        "media/presentation/chips/02-light.svg",
        "4e399c79f4e0bfa56bdf4a86b551191a5534fd499d51de414e5b0d31cd63b3aa",
    ),
    (
        "media/presentation/chips/02-dark.svg",
        "e03c7ac8e5a4a2794180cd22913477ed7e54e5d32dde3066f0089c1cf5465ce3",
    ),
    (
        "media/presentation/chips/03-light.svg",
        "865090b3f865e2682231124d870338099a17ec37e9407e5be3973c8571f079d9",
    ),
    (
        "media/presentation/chips/03-dark.svg",
        "88c6121b08bc8a95f8666d9a02af346ffc0776e5ece865a605e323ef11599c86",
    ),
    (
        "media/presentation/chips/04-light.svg",
        "058fafbbe1bc80d77cd2b24f506674b3e8940b70e40ef2ce7f8500f0506f770d",
    ),
    (
        "media/presentation/chips/04-dark.svg",
        "458ebc39c0b25944f5fe249fa709f7fb011bb4359afc6721c2289d9aa1b91177",
    ),
    (
        "media/presentation/chips/05-light.svg",
        "f6696cfb93f899c380b24988cd3473fd8987f2c71b156752355407e9aae76a70",
    ),
    (
        "media/presentation/chips/05-dark.svg",
        "256f86e09deaee7505000945f5c67eec176fcdfbeebe17666701bbf6d4e9d804",
    ),
    (
        "media/presentation/chips/06-light.svg",
        "2204aa54dee2fe1b9b31d8341ae6caee88532f10616a1fb36bc14ae350d5afb4",
    ),
    (
        "media/presentation/chips/06-dark.svg",
        "6be92d4d5fd3f3474b4e7237ad097989083bbe31c4862c9791b490d3c87e98fd",
    ),
    (
        "media/evidence-lab/03-update-restart-102s.png",
        "d3de2029ce993457d411649bcac5a41f27f1ec1ed9586cb2f9b4dcbbc8675d14",
    ),
    (
        "media/GEL-EVIDENCE-LAB-EN.mp4",
        "a7b4e85d5db1274c61a49d8814908321130dad6c3eec9324de0500071d9c9835",
    ),
    (
        "media/evidence-lab/01-source-17s.png",
        "9c73954fe6b732f56140d7d541e2ba4e04768ced5f3f4ea90036de93b2e240c0",
    ),
    (
        "media/evidence-lab/02-reopened-56s.png",
        "3d385ca2c4862ac873c089be1502770550ea810d5af51d16a15656335e08a82a",
    ),
    (
        "docs/images/q8-four-views-en.png",
        "5cfb026dbed930041b42fb2c692d0e58994bd08b80ec497f3fef254cb1aa20a1",
    ),
    (
        "docs/images/evidence-limits-en.png",
        "6a6b9af2ea626591863d56b6d082613794d52ae1b8f825244ba0b62fecd8b9fd",
    ),
    (
        "media/presentation/badges/commitments-dark.svg",
        "6fee3f55e264bd1b3cfa2604233f3b490a1bff7c3864c524d41a662266d14bde",
    ),
    (
        "media/presentation/badges/commitments-light.svg",
        "430b08526db72bcf73ab1fb4fbae2f5bc96683f5ace09b399fcf7877c58aa8f0",
    ),
    (
        "media/presentation/badges/faster-dark.svg",
        "a235dd89c7ae7206d9f5a6a18b6c24aaf0072593d854f27e7d419e6beca057fa",
    ),
    (
        "media/presentation/badges/faster-light.svg",
        "afee602addf16f856f2daa9b7081227e3b115cc97bc1050b26ccfb24fa7f502c",
    ),
    (
        "media/presentation/badges/license-dark.svg",
        "b88d5aa1514c48f4f92ad46bd53821147c0b55c31d9566bcf890435adf5788dd",
    ),
    (
        "media/presentation/badges/license-light.svg",
        "37be4353432548e02610a1dd8bc6b5505c15b8f811cb60fbb48b0849b4a647c8",
    ),
    (
        "media/presentation/badges/lost-dark.svg",
        "98ed74f8672e132e6eda132cbbc2e26f15d425357d8c89f79aef4c75fd77df9d",
    ),
    (
        "media/presentation/badges/lost-light.svg",
        "d22c5a7d36c0b58c2fdb473226b2d0a58fb783d8341ef84f3f718ff8904cc8d3",
    ),
    (
        "media/presentation/badges/mutants-dark.svg",
        "b774d16618ffe1d7ddfd4b8d28526ee27efa6f2aeccf76ec572e68f773f65a50",
    ),
    (
        "media/presentation/badges/mutants-light.svg",
        "c9d864592601c5d4e9bef8306fe45c94de9fb6f38f8010801d407fb8e76f10a4",
    ),
    (
        "media/presentation/badges/packages-dark.svg",
        "bcbb4c004c58759e37bc70c88a431ae63deafac46a1682c6ff072e2e309fe909",
    ),
    (
        "media/presentation/badges/packages-light.svg",
        "b57511fecd10dbcd404e6bb8c1d0f8e4bbcda2852948ee53718f9b3c79416e95",
    ),
    (
        "media/presentation/badges/version-dark.svg",
        "a40d4924e49ebdfcc3e217b4a21b9f28c6994b524cf98dbc7c85df2d4b75ed28",
    ),
    (
        "media/presentation/badges/version-light.svg",
        "1e74c869bb111ce3bccca8aee6addfa964b140d0c1f17f8b6a418cc1ddcdc578",
    ),
    (
        "media/presentation/badges/views-dark.svg",
        "8b0de64284dd8a9634f347a67f0035a62eaee9da3099874554d1f20eee126bc6",
    ),
    (
        "media/presentation/badges/views-light.svg",
        "0a31425923acd6746f8746a130449723598e786b2d62e428ed446fb74a02b596",
    ),
    (
        "media/presentation/phone/bars-dark.svg",
        "d5826edc082c38cc535989fa830c86b9d1f6c4dcb557147b56d4dfbdb4858a37",
    ),
    (
        "media/presentation/phone/bars-light.svg",
        "500b0ff238f86118c3bd42a2970e04a8dcae5564f47ec0bd3a0f636b816f8f90",
    ),
    (
        "media/presentation/phone/core-dark.svg",
        "fa8420bcda409a427081b5552a6f3b49b7f3f46963bc5ec5c699580b2f7483f0",
    ),
    (
        "media/presentation/phone/core-light.svg",
        "d86001219be70bdb2a602264ca60b8a98828143af1aace2e90d2bbf25c7310ab",
    ),
    (
        "media/presentation/phone/crash-dark.svg",
        "c8da63a85bcafcda3622598495be8652e394b3087f70127b50486cd75e569414",
    ),
    (
        "media/presentation/phone/crash-light.svg",
        "e07abe512bda01379080971d507ba82c10ec21bf526461a453b942d4527bb504",
    ),
    (
        "media/presentation/phone/flow-dark.svg",
        "d8d2a2fcb527229222b93c1531dc1c76ca3a4400d08f0226318e0381bf611ad2",
    ),
    (
        "media/presentation/phone/flow-light.svg",
        "b5134023de67d4d066956dad8b6a3d5ec99c59b7442556483ea6f0ee222f41f0",
    ),
    (
        "media/presentation/phone/glance-dark.svg",
        "d9ffb22c5a63490bee86d4d48904e07ea93ba6f807b041252b7652d059970f66",
    ),
    (
        "media/presentation/phone/glance-light.svg",
        "345522702de0883318f60dd431568b9e80b5b193e1c981ad992a8cea7e426ef1",
    ),
    (
        "media/presentation/phone/header-dark.svg",
        "3bde21e5723ba6f2c0ff7fddd38f3f065719033dd2b96c33e2dc7e3e7e916e23",
    ),
    (
        "media/presentation/phone/header-light.svg",
        "dd6f942b028b7ab48d948a4ae74613709449e1502a470752b7db3d878ea42508",
    ),
    (
        "media/presentation/phone/license-dark.svg",
        "e40bfad73a9031456c36f2394a0adeb284a8cce8eb4e451f04e4c3892c7b3835",
    ),
    (
        "media/presentation/phone/license-light.svg",
        "cc99b41efd30556037210931cc42147bd48594a9cee0e6dba309b2deaa745ff5",
    ),
    (
        "media/presentation/phone/pledges-dark.svg",
        "fde803ca5696442b3407748305511c80fcf6b4f2650dcefa90d6a7b96528366f",
    ),
    (
        "media/presentation/phone/pledges-light.svg",
        "73ac1444d9453b1a0cc1ca873ae05eec1f04fefbf83d8d57dd76f3ba85907a58",
    ),
    (
        "media/presentation/phone/q8-dark.svg",
        "9a75067082ec6ecd4f3d1d3907d23165aca46623a6b9cf892aedf41464714195",
    ),
    (
        "media/presentation/phone/q8-light.svg",
        "706a663b5b7be8f2c9c9844f8b89dc742941ce99b64a9eb5312bdea473914ea0",
    ),
    (
        "media/presentation/phone/record-dark.svg",
        "6b18852be5b4ec6f26f16ef710b628f486235e09baf88b3df8389b68404142f2",
    ),
    (
        "media/presentation/phone/record-light.svg",
        "7dcb5498b3f95ddb07a9dc8c7c26ad27d486f045a8ac25429817b9e319b59b38",
    ),
    (
        "media/presentation/phone/rerun-dark.svg",
        "349c08b880338eedc4b901fbc23455de63ea6c5ebbe1653e6e92a03c65b10e39",
    ),
    (
        "media/presentation/phone/rerun-light.svg",
        "a6b668ee0cd7ebbb3c1dae93b4b70e9d3a44d8e0c179cf4215e4b00f49e82a17",
    ),
    (
        "media/presentation/phone/speed-dark.svg",
        "d2450a8d7a9b612f16c962e586f3f4f83680bf4d0b2327fb1cf2cd82fa3b94a5",
    ),
    (
        "media/presentation/phone/speed-light.svg",
        "7ff94b9b5dfb3963e19149f93d101091d029cddedb4e14ff492f3f362b36c595",
    ),
    (
        "media/presentation/phone/status-dark.svg",
        "7541d433779df2e531392ce144095e17427597a7061ee2c604204781145d06d1",
    ),
    (
        "media/presentation/phone/status-light.svg",
        "71381f5a909561a99882257908dfd22933bcf2c5175f5cc8e17f39b4b6669029",
    ),
    (
        "media/presentation/phone/terminal-dark.svg",
        "5d41f5291c698d0ae2e87d6b8426d33ec27342c465f79947660b378b893fabdf",
    ),
    (
        "media/presentation/phone/terminal-light.svg",
        "9eb561f342c782d3c36dc4d5d2f8af932b98d3ed760d1378e5a2cb2249b60a24",
    ),
    (
        "media/presentation/phone/wall-dark.svg",
        "9539a2b6a9f5fd94a396df6eb3ca1b18f768c85e257f5183a458980e005501e8",
    ),
    (
        "media/presentation/phone/wall-light.svg",
        "b4def92c98015541efbf47c4658e2cc5a0f17b89fcbe8c7d4f66ef716c0be603",
    ),
    (
        "media/presentation/q8-dark.svg",
        "5e1817e133c443482c392a9aea5be6b1d31f4c2e44f096b20c6d6f444e5ae5f0",
    ),
    (
        "media/presentation/q8-light.svg",
        "a81597ef68d792bb93d09af295cd9bcda91a33e6f4402d0ee9c530b7a25ca456",
    ),
];

/// The ticked CLA acknowledgement line from `.github/PULL_REQUEST_TEMPLATE.md`.
const CLA_ACK_TICKED: &[&str] = &[
    "[x] I have completed the GEL RAM CLA privately with the project before opening this pull request.",
    "[X] I have completed the GEL RAM CLA privately with the project before opening this pull request.",
];

const USAGE: &str =
    "verify|report|reproduce|witness|rebuild-check|isolation-check|mutation-matrix|history-mutation-matrix|mutation-campaign|memory-run|memory-check|bench-compare|speed-ab|speed-ab-check|speed-campaign|package-binaries|ci-evidence|ci-evidence-import|ci-evidence-recorded|claims|roadmap|crash-series|history-crash-series|runtime-examples|source-audit|source-bundle|source-headers|ucd-tables|commitment|commitment-check|rust-only|licensing|ci-policy|docs-refs|cla-ack|fmt|clippy|recorder-lint|platform-diff|test|bench|physics";
const CHECKOUT_SHA: &str = "actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1";

// Exact active-license text pin; not evidence of legal approval or authorship.
fn canonical_license(bytes: &[u8]) -> bool {
    gel_source::hex(&gel_source::digest(bytes))
        == "c0b560ffc53ad735c4a6236356ff0cd47d92cf671181e799917975b071e8cd40"
}

const FMT_CHECK_ARGS: &[&str] = &["fmt", "--all", "--", "--check"];
const CLIPPY_ARGS: &[&str] = &[
    "clippy",
    "--locked",
    "--offline",
    "--workspace",
    "--all-targets",
    "--",
    "-D",
    "warnings",
];
const TEST_ARGS: &[&str] = &[
    "test",
    "--locked",
    "--offline",
    "--workspace",
    "--all-targets",
];

/// The walk of `root` that every whole-tree gate reads, with its error as text.
fn walk(root: &Path) -> Result<tree_scan::Tree, String> {
    tree_scan::walk(root).map_err(|e| e.to_string())
}

/// Prints a gate report (standard output lines, then standard error lines) and
/// returns its result.
fn emit(report: tree_scan::Report) -> Result<(), String> {
    for line in &report.stdout {
        println!("{line}");
    }
    for line in &report.stderr {
        eprintln!("{line}");
    }
    report.result
}

fn workspace_root() -> Result<&'static Path, String> {
    // Resolve the source checkout embedded by this build, not the caller's cwd.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| "missing workspace root".into())
}

fn rust_only_at(root: &Path) -> Result<(), String> {
    rust_only_in(root, &walk(root)?)
}

fn rust_only_in(root: &Path, tree: &tree_scan::Tree) -> Result<(), String> {
    emit(tree_scan::rust_only(root, tree, REVIEWED_ASSETS))
}

fn disclosure() -> Result<(), String> {
    let root = workspace_root()?;
    disclosure::check(root, &walk(root)?.paths)
}

fn rust_only() -> Result<(), String> {
    rust_only_at(workspace_root()?)
}

fn require(text: &str, needle: &str, file: &str) -> Result<(), String> {
    if text.contains(needle) {
        Ok(())
    } else {
        Err(format!("{file} is missing required text: {needle}"))
    }
}

// Technical tests must run before publication approval; PASS is not permission.
fn publication_status(text: &str) -> Result<bool, String> {
    let flag = |key: &str| -> Result<bool, String> {
        let values: Vec<_> = text
            .lines()
            .filter_map(|line| {
                let (name, value) = line.trim().split_once('=')?;
                (name == key).then_some(value)
            })
            .collect();
        match values.as_slice() {
            ["YES"] => Ok(true),
            ["NO"] => Ok(false),
            _ => Err(format!("missing, duplicated or invalid {key}")),
        }
    };
    let review = flag("REVIEW_PUBLICATION_APPROVED")?;
    let publication = flag("PUBLICATION_APPROVED")?;
    if publication && !review {
        return Err("publication cannot precede scope review".into());
    }
    Ok(publication)
}

fn licensing() -> Result<(), String> {
    licensing_with(workspace_root()?, None)
}

/// The licensing gate; `walked` is the tree walk when the caller already has
/// one, otherwise the walk happens where the gate first needs it.
fn licensing_with(root: &Path, walked: Option<&tree_scan::Tree>) -> Result<(), String> {
    let read = |name: &str| fs::read_to_string(root.join(name)).map_err(|e| format!("{name}: {e}"));
    let mode = read("docs/LICENSE-MODE.txt")?;
    let license = read("LICENSE")?;
    let notice = read("NOTICE")?;
    let commercial = read("COMMERCIAL-LICENSE.md")?;
    let cla = read("CLA.md")?;
    licensing_scope(&read("LICENSING.md")?, &notice)?;
    let fixture_license = fs::read(root.join("crates/gel-source/fixtures/rust-book/LICENSE.txt"))
        .map_err(|e| e.to_string())?;
    if gel_source::hex(&gel_source::digest(&fixture_license))
        != "0621878e61f0d0fda054bcbe02df75192c28bde1ecc8289cbd86aeba2dd72720"
    {
        return Err("reviewed third-party fixture license is missing or changed".into());
    }
    let owned;
    let tree = match walked {
        Some(tree) => tree,
        None => {
            owned = walk(root)?;
            &owned
        }
    };
    license_metadata::check(root, tree)?;
    if !canonical_license(license.as_bytes()) {
        return Err("LICENSE differs from the pinned active NCRL bytes".into());
    }
    match mode.trim() {
        "GEL-RAM-NCRL-1.0 + Commercial + CLA-2.0" => {
            require(
                &license,
                "GEL RAM Noncommercial Reciprocal License 1.0",
                "LICENSE",
            )?;
            require(
                &notice,
                "Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)",
                "NOTICE",
            )?;
        }
        other => return Err(format!("unsupported docs/LICENSE-MODE.txt value: {other}")),
    }
    require(
        &commercial,
        "not itself a commercial license grant",
        "COMMERCIAL-LICENSE.md",
    )?;
    for text in [
        "Contributor License Agreement 2.0",
        "right to license or relicense",
        "Patent license",
        "legal authority",
        "third-party material",
        "AI system",
        "completed Agreement is on file",
        "Pin the exact accepted agreement text",
        "public pseudonym",
        "CLA-PRIVACY.md",
    ] {
        require(&cla, text, "CLA.md")?;
    }
    require(
        &read("CLA-PRIVACY.md")?,
        "RR is a public pseudonym",
        "CLA-PRIVACY.md",
    )?;
    let publication = publication_status(&read("CANDIDATE-STATUS.md")?)?;
    println!(
        "PUBLICATION_APPROVED={}",
        if publication { "YES" } else { "NO" }
    );
    require(
        &read("CANDIDATE-STATUS.md")?,
        "LEGAL_APPROVED=NO",
        "CANDIDATE-STATUS.md",
    )?;
    emit(tree_scan::project_contact(tree))?;
    println!("LICENSING_GATE=PASS");
    println!("LICENSE_MODE={}", mode.trim());
    println!("LICENSE_CHECK_SCOPE=ACTIVE_LICENSE_CONSISTENCY_NOT_LEGAL_CERTIFICATION");
    Ok(())
}

fn licensing_scope(guide: &str, notice: &str) -> Result<(), String> {
    require(
        guide,
        "Identified third-party material remains under its own license terms",
        "LICENSING.md",
    )?;
    require(guide, "remains MIT-licensed", "LICENSING.md")?;
    require(notice, "The Rust Book excerpt", "NOTICE")?;
    require(notice, "distributed under MIT", "NOTICE")?;
    let normalized = guide
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    for obsolete in [
        "everything in this distribution is",
        "no other public license is granted for this tree",
        "this tree is offered under polyform noncommercial 1.0.0 only",
    ] {
        if normalized.contains(obsolete) {
            return Err(
                "overbroad whole-tree licensing claim conflicts with third-party material".into(),
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod licensing_scope_tests {
    const GUIDE: &str = include_str!("../../LICENSING.md");
    const NOTICE: &str = include_str!("../../NOTICE");
    #[test]
    fn root_third_party_scope_and_notices() {
        super::licensing_scope(GUIDE, NOTICE).unwrap();
        assert!(
            super::licensing_scope(&GUIDE.replace("remains MIT-licensed", "omitted"), NOTICE)
                .is_err()
        );
        assert!(
            super::licensing_scope(GUIDE, &NOTICE.replace("distributed under MIT", "omitted"))
                .is_err()
        );
    }
    #[test]
    fn appended_overbroad_claim_is_rejected() {
        for claim in [
            "Everything in this distribution is ours.",
            "No other public license is granted for this tree.",
            "This tree is offered under PolyForm Noncommercial 1.0.0 only.",
        ] {
            assert!(super::licensing_scope(&format!("{GUIDE}\n{claim}"), NOTICE).is_err());
        }
    }
}

fn ci_policy() -> Result<(), String> {
    let root = workspace_root()?;
    let ci = fs::read_to_string(root.join(".github/workflows/ci.yml"))
        .map_err(|e| format!(".github/workflows/ci.yml: {e}"))?;
    let cla = fs::read_to_string(root.join(".github/workflows/cla.yml"))
        .map_err(|e| format!(".github/workflows/cla.yml: {e}"))?;
    require(&ci, CHECKOUT_SHA, ".github/workflows/ci.yml")?;
    require(
        &ci,
        "persist-credentials: false",
        ".github/workflows/ci.yml",
    )?;
    require(&ci, "contents: read", ".github/workflows/ci.yml")?;
    require(&cla, "pull_request_target:", ".github/workflows/cla.yml")?;
    require(&cla, "permissions: {}", ".github/workflows/cla.yml")?;
    require(
        &cla,
        "github.event.pull_request.author_association",
        ".github/workflows/cla.yml",
    )?;
    require(
        &cla,
        "repository owner is the Project Licensor",
        ".github/workflows/cla.yml",
    )?;
    require(&cla, "grep -Fqx", ".github/workflows/cla.yml")?;
    if cla.contains("actions/checkout") || cla.contains("cargo run") {
        return Err(
            ".github/workflows/cla.yml must not check out or execute pull-request code".into(),
        );
    }
    let binaries_path = ".github/workflows/binaries.yml";
    let binaries = fs::read_to_string(root.join(binaries_path))
        .map_err(|e| format!("{binaries_path}: {e}"))?;
    require(&binaries, CHECKOUT_SHA, binaries_path)?;
    require(&binaries, "persist-credentials: false", binaries_path)?;
    require(&binaries, "contents: read", binaries_path)?;
    // Only the hand-started release job may request a signing identity.
    if binaries.matches("id-token: write").count() != 1
        || !binaries.contains("if: github.event_name == 'workflow_dispatch'\n")
    {
        return Err(format!(
            "{binaries_path}: id-token must be granted once, to the workflow_dispatch job"
        ));
    }
    // Every workflow pins its actions to commits. Only the hand-started release
    // job of the binaries workflow may hold a write permission; no workflow
    // writes to the repository itself.
    let dir = root.join(".github/workflows");
    let workflows = audit_io::paths(&dir)?;
    for path in &workflows {
        let metadata = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
        if !metadata.is_file()
            || !matches!(
                path.extension().and_then(OsStr::to_str),
                Some("yml" | "yaml")
            )
        {
            return Err("workflow inventory contains a non-regular or unsupported entry".into());
        }
        let file = format!(
            ".github/workflows/{}",
            path.file_name()
                .and_then(OsStr::to_str)
                .ok_or("non-UTF8 workflow name")?
        );
        let text = fs::read_to_string(path).map_err(|e| format!("{file}: {e}"))?;
        workflow_policy::check(&text, &file)?;
    }
    println!("CI_POLICY_GATE=PASS workflows={}", workflows.len());
    Ok(())
}

fn docs_refs() -> Result<(), String> {
    let root = workspace_root()?;
    emit(tree_scan::docs_refs(root, &walk(root)?))
}

/// `source-headers`: the gate; `source-headers --write`: insert missing headers.
fn source_headers(args: &[String]) -> Result<(), String> {
    let root = workspace_root()?;
    match args {
        [] => emit(source_headers::check(root, &walk(root)?)),
        [flag] if flag == "--write" => emit(source_headers::write(root, &walk(root)?)),
        _ => Err("usage: cargo run -p xtask -- source-headers [--write]".into()),
    }
}

fn cla_acknowledged(body: &str) -> bool {
    // Match the workflow's grep -Fqx contract: one complete checkbox line.
    body.split('\n').any(|line| {
        CLA_ACK_TICKED
            .iter()
            .any(|ack| line.strip_prefix("- ") == Some(*ack))
    })
}

fn cla_ack() -> Result<(), String> {
    let body = std::env::var_os("PR_BODY")
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_default();
    if cla_acknowledged(&body) {
        println!("CLA_ACK_GATE=PASS");
        Ok(())
    } else {
        eprintln!(
            "PR_BODY is unset or does not contain the ticked CLA acknowledgement line from .github/PULL_REQUEST_TEMPLATE.md."
        );
        eprintln!(
            "A completed CLA must be on file with the project before a pull request is opened; see CLA.md and CONTRIBUTING.md."
        );
        Err("CLA_ACK_GATE=FAIL".into())
    }
}

fn run(program: &str, args: &[&str]) -> Result<(), String> {
    let mut command = Command::new(program);
    command.args(args).current_dir(workspace_root()?);
    process_sequence::sequence(&mut [command])
}

fn runtime_examples() -> Result<(), String> {
    let root = workspace_root()?;
    let specs: &[&[&str]] = &[
        &["-p", "gel-source", "--example", "source_build"],
        &["-p", "gel-live-lab", "--", "--demo"],
        &["-p", "gel-live-lab", "--", "--literal", "README.md"],
        &["-p", "gel-live-lab", "--", "--witness"],
        &[
            "-p",
            "gel-live-lab",
            "--bin",
            "gel-evidence",
            "--",
            "--demo",
        ],
        &["-p", "gel-cli", "--example", "quantization_matrix"],
        &["-p", "gel-cli", "--example", "precision_matrix"],
        &["-p", "gel-source", "--example", "collection_review"],
        &["-p", "gel-history", "--example", "record_history"],
    ];
    let mut commands = Vec::new();
    for spec in specs {
        let mut command = Command::new("cargo");
        command
            .args(["run", "--locked", "--offline", "--release"])
            .args(*spec)
            .current_dir(root);
        commands.push(command);
    }
    process_sequence::sequence(&mut commands)?;
    println!("RUNTIME_EXAMPLES=PASS");
    Ok(())
}

fn run_docs() -> Result<(), String> {
    let status = Command::new("cargo")
        .args(["doc", "--locked", "--offline", "--workspace", "--no-deps"])
        .env("RUSTDOCFLAGS", "-D warnings")
        .current_dir(workspace_root()?)
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("cargo doc failed: {status}"))
    }
}

/// `cargo run --release -p <package> -- <args>` inside the workspace.
fn run_release_binary<S: AsRef<OsStr>>(package: &str, args: &[S]) -> Result<(), String> {
    let mut command = Command::new("cargo");
    command.args([
        "run",
        "--locked",
        "--offline",
        "--release",
        "-p",
        package,
        "--",
    ]);
    command.args(args);
    let status = command
        .current_dir(workspace_root()?)
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{package} failed: {status}"))
    }
}

/// Release examples that `verify` runs. Its one release build compiles them together
/// with every library and binary, so each later `cargo run` step only executes.
const VERIFY_RELEASE_EXAMPLES: &[(&str, &str)] = &[
    ("gel-source", "collection_recheck"),
    ("gel-phase-quad", "quad_compare"),
    ("gel-phase-quad", "quad_evidence"),
    ("gel-phase-quad", "quad_playground"),
    ("gel-source", "source_readout"),
    ("gel-source", "source_parts"),
    ("gel-source", "source_real"),
    ("gel-source", "source_find"),
    ("gel-source", "source_build"),
    ("gel-source", "collection_review"),
    ("gel-cli", "quantization_matrix"),
    ("gel-history", "record_history"),
];

/// Examples that `verify` runs one after another, in order, with their arguments.
const VERIFY_LOOP_EXAMPLES: &[(&str, &str, &[&str])] = &[
    ("gel-phase-quad", "quad_playground", &["--demo"]),
    ("gel-source", "source_readout", &[]),
    ("gel-source", "source_parts", &[]),
    ("gel-source", "source_real", &[]),
    ("gel-source", "source_find", &[]),
    ("gel-source", "source_build", &[]),
    ("gel-source", "collection_review", &[]),
    ("gel-cli", "quantization_matrix", &[]),
    ("gel-history", "record_history", &[]),
];

/// `cargo build --release` of every library and binary (the default targets) plus
/// every entry of `VERIFY_RELEASE_EXAMPLES`, in one cargo invocation.
fn verify_release_build_args() -> Vec<&'static str> {
    let mut args = vec![
        "build",
        "--locked",
        "--offline",
        "--release",
        "--workspace",
        "--lib",
        "--bins",
    ];
    for (_, example) in VERIFY_RELEASE_EXAMPLES {
        args.extend(["--example", example]);
    }
    args
}

/// `cargo run --release -p <package> --example <example> -- <args>` for an example
/// that the verify release build has already compiled.
fn release_example_args<'a>(
    package: &'a str,
    example: &'a str,
    args: &[&'a str],
) -> Result<Vec<&'a str>, String> {
    if !VERIFY_RELEASE_EXAMPLES
        .iter()
        .any(|&(p, e)| p == package && e == example)
    {
        return Err(format!(
            "{package} example {example} is not in the verify release build"
        ));
    }
    let mut command = vec![
        "run",
        "--locked",
        "--offline",
        "--release",
        "-p",
        package,
        "--example",
        example,
        "--",
    ];
    command.extend_from_slice(args);
    Ok(command)
}

fn verify() -> Result<(), String> {
    // One walk for every gate that reads the whole tree; none of these gates
    // writes to the tree.
    let root = workspace_root()?;
    let tree = walk(root)?;
    rust_only_in(root, &tree)?;
    licensing_with(root, Some(&tree))?;
    ci_policy()?;
    emit(tree_scan::docs_refs(root, &tree))?;
    disclosure::check(root, &tree.paths)?;
    let headers = source_headers::check(root, &tree);
    let header_files = source_headers::passed_files(&headers);
    emit(headers)?;
    let header_files =
        header_files.ok_or("source headers: the gate printed no PASS line with a file count")?;
    speed_convention::check(root)?;
    run("cargo", FMT_CHECK_ARGS)?;
    run("cargo", CLIPPY_ARGS)?;
    recorder_lint::check(workspace_root()?)?;
    run("cargo", &verify_release_build_args())?;
    license_self::check(workspace_root()?, header_files)?;
    run_docs()?;
    measured_sources::verify(workspace_root()?)?;
    claims::check(workspace_root()?)?;
    answer_bench::check(workspace_root()?)?;
    roadmap::check(workspace_root()?)?;
    ci_evidence::recorded(&[])?;
    if cfg!(unix) {
        let report = crash_series::run(&gel_evidence_binary(workspace_root()?), 5, 20_260_929)?;
        println!("{}", report.lines().last().unwrap_or(""));
        let report = history_crash_series::run(5, 20_261_010, 0)?;
        println!("{}", report.lines().last().unwrap_or(""));
    } else {
        println!("CRASH_SERIES=SKIPPED not a Unix host");
        println!("HISTORY_CRASH_SERIES=SKIPPED not a Unix host");
    }
    mutation_matrix::check(workspace_root()?)?;
    history_mutation_matrix::check(workspace_root()?)?;
    println!(
        "PROPERTY_MAP_FORMAT=PASS rows={}",
        property_map::read(workspace_root()?)?.len()
    );
    run(
        "cargo",
        &["test", "--locked", "--offline", "--workspace", "--doc"],
    )?;
    run(
        "cargo",
        &release_example_args(
            "gel-source",
            "collection_recheck",
            &["docs/evidence-collection/r1"],
        )?,
    )?;
    run("cargo", TEST_ARGS)?;
    run_release_binary("gel-cli", &["selftest"])?;
    run_release_binary("gel-bench", &["8192", "3", "2"])?;
    run_release_binary("gel-bench", &["8192", "3", "1"])?;
    run(
        "cargo",
        &release_example_args(
            "gel-phase-quad",
            "quad_compare",
            &[
                "--orbs",
                "32",
                "--rounds",
                "3",
                "--workers",
                "2",
                "--sparse",
                "1",
            ],
        )?,
    )?;
    run(
        "cargo",
        &release_example_args("gel-phase-quad", "quad_evidence", &["--demo"])?,
    )?;
    for (package, example, args) in VERIFY_LOOP_EXAMPLES {
        run("cargo", &release_example_args(package, example, args)?)?;
    }
    run(
        "cargo",
        &[
            "run",
            "--locked",
            "--offline",
            "--release",
            "-p",
            "gel-live-lab",
            "--",
            "--demo",
        ],
    )?;
    run(
        "cargo",
        &[
            "run",
            "--locked",
            "--offline",
            "--release",
            "-p",
            "gel-live-lab",
            "--bin",
            "gel-evidence",
            "--",
            "--demo",
        ],
    )?;
    println!("GEL_VERIFY_ALL=PASS");
    Ok(())
}

fn collect_args() -> Result<Vec<String>, String> {
    std::env::args_os()
        .skip(1)
        .map(|arg| {
            arg.into_string()
                .map_err(|bad| format!("argument is not valid UTF-8: {}", bad.to_string_lossy()))
        })
        .collect()
}

fn dispatch(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        None | Some("verify") => verify(),
        Some("report") => reproduce::report(&args[1..]),
        Some("reproduce") => reproduction::run(&args[1..]),
        Some("witness") => witness::run(&args[1..]),
        Some("rebuild-check") => rebuild_check::run(&args[1..]),
        Some("isolation-check") => isolation::check(&args[1..]),
        Some("mutation-matrix") => mutation_matrix::run(&args[1..]),
        Some("history-mutation-matrix") => history_mutation_matrix::run(&args[1..]),
        Some("mutation-campaign") => mutation_campaign::run(&args[1..]),
        Some("memory-run") => memory::run(&args[1..]),
        Some("memory-check") => memory::check_cmd(&args[1..]),
        Some("memory-touch") => memory::touch(&args[1..]),
        Some("answer-bench") => answer_bench::run(&args[1..]),
        Some("bench-compare") => bench_compare::run(&args[1..]),
        Some("speed-ab") => speed_ab::run(&args[1..]),
        Some("speed-ab-check") => speed_campaign::check_cmd(&args[1..]),
        Some("speed-campaign") => speed_campaign::campaign_cmd(&args[1..]),
        Some("package-binaries") => package::run(&args[1..]),
        Some("ci-evidence") => ci_evidence::report(&args[1..]),
        Some("ci-evidence-import") => ci_evidence::import(&args[1..]),
        Some("ci-evidence-recorded") => ci_evidence::recorded(&args[1..]),
        Some("claims") => claims::check(workspace_root()?),
        Some("roadmap") => roadmap::check(workspace_root()?),
        Some("crash-series") => crash_series_cmd(&args[1..]),
        Some("history-crash-series") => history_crash_series_cmd(&args[1..]),
        Some("history-writer") => history_crash_series::writer(&args[1..]),
        Some("runtime-examples") => runtime_examples(),
        Some("source-audit") => source_bundle::audit(&args[1..]),
        Some("source-bundle") => source_bundle::bundle(&args[1..]),
        Some("source-headers") => source_headers(&args[1..]),
        Some("ucd-tables") => ucd_tables::run(&args[1..]),
        Some("commitment") => commitment::create(&args[1..]),
        Some("commitment-check") => commitment::check(&args[1..]),
        Some("rust-only") => rust_only(),
        Some("licensing") => licensing(),
        Some("ci-policy") => ci_policy(),
        Some("docs-refs") => docs_refs(),
        Some("disclosure") => disclosure(),
        Some("speed-convention") => speed_convention::check(workspace_root()?),
        Some("cla-ack") => cla_ack(),
        Some("fmt") => run("cargo", FMT_CHECK_ARGS),
        Some("clippy") => run("cargo", CLIPPY_ARGS),
        Some("recorder-lint") => recorder_lint::check(workspace_root()?),
        Some("platform-diff") => ci_evidence::platform_diff(&args[1..]),
        Some("test") => run("cargo", TEST_ARGS),
        Some("bench") => run_release_binary("gel-bench", &args[1..]),
        Some("physics") => run_release_binary("gel-physics", &args[1..]),
        Some(x) => Err(format!("unknown xtask: {x}; use {USAGE}")),
    }
}

fn main() -> ExitCode {
    match collect_args().and_then(|args| dispatch(&args)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree_scan::{
        code_spans, has_nonproject_gmail, is_reference, reference_tokens, strip_span_padding,
        PROJECT_EMAIL,
    };

    #[test]
    fn component_evidence_exception_is_exact_and_text_only() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("gel-component-gate-{}-{nonce}", std::process::id()));
        let dir = root.join("docs/evidence-gel-components");
        fs::create_dir_all(&dir).unwrap();
        for i in 1..=3 {
            let path = dir.join(format!("run-{i}.csv"));
            fs::write(&path, b"rep,n\n0,16384\n").unwrap();
            assert!(rust_only_at(&root).is_ok());
            for bad in [b"\xff".as_slice(), b"\x1b[2J", b"binary\0"] {
                fs::write(&path, bad).unwrap();
                assert!(rust_only_at(&root).is_err());
            }
            fs::write(&path, b"rep,n\n0,16384\n").unwrap();
        }
        fs::write(dir.join("run-4.csv"), b"unreviewed\n").unwrap();
        assert!(rust_only_at(&root).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn collection_evidence_exception_is_exact_and_text_only() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("gel-evidence-gate-{}-{nonce}", std::process::id()));
        fs::create_dir_all(root.join("docs/evidence-collection/r1")).unwrap();
        let raw = root.join("docs/evidence-collection/r1/raw.csv");
        fs::write(&raw, b"rep,documents\n0,8\n").unwrap();
        assert!(rust_only_at(&root).is_ok());
        for bad in [b"\xff".as_slice(), b"\x1b[2J", b"binary\0"] {
            fs::write(&raw, bad).unwrap();
            assert!(rust_only_at(&root).is_err());
        }
        fs::write(&raw, b"rep,documents\n0,8\n").unwrap();
        fs::write(root.join("unreviewed.csv"), b"text\n").unwrap();
        assert!(rust_only_at(&root).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn documentation_png_gate_requires_exact_reviewed_bytes_and_path() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("gel-image-gate-{}-{nonce}", std::process::id()));
        fs::create_dir_all(root.join("docs/images")).unwrap();
        for (name, _) in REVIEWED_ASSETS {
            let bytes = fs::read(workspace_root().unwrap().join(name)).unwrap();
            let path = root.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, &bytes).unwrap();
            assert!(rust_only_at(&root).is_ok());
            let mut changed = bytes.clone();
            changed[100] ^= 1;
            fs::write(&path, &changed).unwrap();
            assert!(rust_only_at(&root).is_err());
            fs::write(&path, &bytes).unwrap();
            let unexpected = root.join("docs/images/unreviewed.png");
            fs::write(&unexpected, &bytes).unwrap();
            assert!(rust_only_at(&root).is_err());
            fs::remove_file(unexpected).unwrap();
            fs::write(&path, b"not a PNG").unwrap();
            assert!(rust_only_at(&root).is_err());
            fs::remove_file(path).unwrap();
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn canonical_license_rejects_changed_or_appended_bytes() {
        let original = include_bytes!("../../LICENSE");
        assert!(canonical_license(original));
        let mut changed = original.to_vec();
        changed[0] ^= 1;
        assert!(!canonical_license(&changed));
        changed = original.to_vec();
        changed.push(b'\n');
        assert!(!canonical_license(&changed));
    }

    #[test]
    fn contact_gate_distinguishes_source_suffix_from_an_address() {
        assert!(!has_nonproject_gmail("token.ends_with(\"@gmail.com\")"));
        assert!(!has_nonproject_gmail(PROJECT_EMAIL));
        assert!(!has_nonproject_gmail(&format!("<mailto:{PROJECT_EMAIL}>.")));
        assert!(!has_nonproject_gmail(&PROJECT_EMAIL.to_uppercase()));
        for local in ["fixture", "test.user", "test+tag", "test_user"] {
            assert!(has_nonproject_gmail(&format!("`{local}@{}.`", "gmail.com")));
        }
    }

    #[cfg(unix)]
    #[test]
    fn rust_only_gate_rejects_symlinks_and_executable_files() {
        use std::os::unix::fs::{symlink, PermissionsExt};
        use std::time::{SystemTime, UNIX_EPOCH};

        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("gel-rust-only-gate-{}-{nonce}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let source = root.join("good.rs");
        fs::write(&source, "fn main() {}\n").unwrap();
        fs::write(
            root.join("CITATION.cff"),
            "cff-version: 1.2.0\ntitle: \"fixture\"\n",
        )
        .unwrap();
        assert!(rust_only_at(&root).is_ok());

        let link = root.join("link.rs");
        symlink("good.rs", &link).unwrap();
        let error = rust_only_at(&root).unwrap_err();
        assert_eq!(error, "RUST_ONLY_GATE=FAIL");
        fs::remove_file(&link).unwrap();

        fs::set_permissions(&source, fs::Permissions::from_mode(0o755)).unwrap();
        let error = rust_only_at(&root).unwrap_err();
        assert_eq!(error, "RUST_ONLY_GATE=FAIL");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn code_spans_follow_backtick_run_width() {
        assert_eq!(
            code_spans("see `a.md` and ``b `x` c`` here"),
            vec!["a.md", "b `x` c"]
        );
        assert_eq!(code_spans("`` unmatched ` one.md `"), vec![" one.md "]);
        assert_eq!(code_spans("no spans ` here"), Vec::<&str>::new());
        assert_eq!(strip_span_padding(" one.md "), "one.md");
        assert_eq!(strip_span_padding("  "), "  ");
    }

    #[test]
    fn cla_ack_requires_the_exact_workflow_line() {
        for ack in CLA_ACK_TICKED {
            let line = format!("- {ack}");
            assert!(cla_acknowledged(&format!("Context\n{line}\n")));
            for altered in [
                format!("quote {line}"),
                format!("{line} not true"),
                format!("{line}\r"),
                ack.to_string(),
            ] {
                assert!(!cla_acknowledged(&altered));
            }
        }
        assert!(!cla_acknowledged(""));
    }

    #[test]
    fn fenced_blocks_are_not_scanned() {
        let text = "```text\n`docs/inside.md`\n```\n`docs/outside.md`\n````\n```\n`docs/still-inside.md`\n````\n`CLA.md`\n";
        assert_eq!(
            reference_tokens(text),
            vec![(4, "docs/outside.md"), (9, "CLA.md")]
        );
    }

    #[test]
    fn reference_shape_matches_repository_paths_only() {
        for yes in [
            "CLA.md",
            "docs/",
            ".github/workflows/ci.yml",
            "crates/gel-core/src/lib.rs",
            "xtask/Cargo.toml",
            "Cargo.lock",
            "rust-toolchain.toml",
            "LICENSE-MODE.txt",
            "LICENSE",
            "NOTICE",
            ".gitignore",
        ] {
            assert!(is_reference(yes), "{yes}");
        }
        for no in [
            "",
            ".gel",
            "xtask",
            "gel-bench",
            "u64::count_ones()",
            "gelram.licensing@gmail.com",
            "<ORB_COUNT> <ROUNDS> [THREADS]",
            "docs/<name>.md",
            "record_count * 128",
            "GEL_BENCH_V3",
            "unsafe_code = \"forbid\"",
            "$HOME/x.md",
            "{root}/x.md",
            "x86_64-unknown-linux-gnu",
        ] {
            assert!(!is_reference(no), "{no}");
        }
    }

    #[test]
    fn cla_ack_needles_match_the_template_line() {
        for needle in CLA_ACK_TICKED {
            assert!(needle.starts_with("[x] ") || needle.starts_with("[X] "));
            assert!(needle.ends_with(" before opening this pull request."));
        }
        assert_eq!(CLA_ACK_TICKED[0][4..], CLA_ACK_TICKED[1][4..]);
    }
}

fn gel_evidence_binary(root: &Path) -> PathBuf {
    std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("target"))
        .join("release")
        .join(format!("gel-evidence{}", std::env::consts::EXE_SUFFIX))
}

/// `crash-series [TRIALS] [SEED]`: builds the release `gel-evidence` and prints the full report.
fn crash_series_cmd(args: &[String]) -> Result<(), String> {
    let number = |i: usize, default: u64| -> Result<u64, String> {
        args.get(i)
            .map_or(Ok(default), |s| s.parse().map_err(|e| format!("{s}: {e}")))
    };
    let (trials, seed) = (number(0, 200)? as usize, number(1, 20_260_929)?);
    run(
        "cargo",
        &[
            "build",
            "--locked",
            "--offline",
            "--release",
            "-p",
            "gel-live-lab",
            "--bin",
            "gel-evidence",
        ],
    )?;
    print!(
        "{}",
        crash_series::run(&gel_evidence_binary(workspace_root()?), trials, seed)?
    );
    Ok(())
}

/// `history-crash-series [TRIALS] [SEED] [CONTROL_TRIALS]`: kills the
/// `history-writer` of this binary and prints the full report.
fn history_crash_series_cmd(args: &[String]) -> Result<(), String> {
    let number = |i: usize, default: u64| -> Result<u64, String> {
        args.get(i)
            .map_or(Ok(default), |s| s.parse().map_err(|e| format!("{s}: {e}")))
    };
    let (trials, seed, control) = (
        number(0, 200)? as usize,
        number(1, 20_261_010)?,
        number(2, 20)? as usize,
    );
    print!("{}", history_crash_series::run(trials, seed, control)?);
    Ok(())
}

#[cfg(test)]
mod verify_release_build_tests {
    use super::*;

    /// The release build `verify` ran before every example was compiled in it.
    const REFERENCE_BUILD: &[&str] =
        &["build", "--locked", "--offline", "--release", "--workspace"];

    /// The literal `cargo run` argument lists `verify` used for its examples, in order.
    fn reference_runs() -> Vec<Vec<&'static str>> {
        let mut runs: Vec<Vec<&'static str>> = vec![
            vec![
                "run",
                "--locked",
                "--offline",
                "--release",
                "-p",
                "gel-source",
                "--example",
                "collection_recheck",
                "--",
                "docs/evidence-collection/r1",
            ],
            vec![
                "run",
                "--locked",
                "--offline",
                "--release",
                "-p",
                "gel-phase-quad",
                "--example",
                "quad_compare",
                "--",
                "--orbs",
                "32",
                "--rounds",
                "3",
                "--workers",
                "2",
                "--sparse",
                "1",
            ],
            vec![
                "run",
                "--locked",
                "--offline",
                "--release",
                "-p",
                "gel-phase-quad",
                "--example",
                "quad_evidence",
                "--",
                "--demo",
            ],
        ];
        for (package, example, args) in [
            ("gel-phase-quad", "quad_playground", vec!["--demo"]),
            ("gel-source", "source_readout", vec![]),
            ("gel-source", "source_parts", vec![]),
            ("gel-source", "source_real", vec![]),
            ("gel-source", "source_find", vec![]),
            ("gel-source", "source_build", vec![]),
            ("gel-source", "collection_review", vec![]),
            ("gel-cli", "quantization_matrix", vec![]),
            ("gel-history", "record_history", vec![]),
        ] {
            runs.push(reference_run(package, example, args));
        }
        runs
    }

    /// The loop body `verify` used to build one example command.
    fn reference_run<'a>(package: &'a str, example: &'a str, args: Vec<&'a str>) -> Vec<&'a str> {
        let mut command = vec![
            "run",
            "--locked",
            "--offline",
            "--release",
            "-p",
            package,
            "--example",
            example,
            "--",
        ];
        command.extend(args);
        command
    }

    /// The same example commands as `verify` now composes them, in the same order.
    fn current_runs() -> Vec<Vec<&'static str>> {
        let mut runs = vec![
            release_example_args(
                "gel-source",
                "collection_recheck",
                &["docs/evidence-collection/r1"],
            )
            .unwrap(),
            release_example_args(
                "gel-phase-quad",
                "quad_compare",
                &[
                    "--orbs",
                    "32",
                    "--rounds",
                    "3",
                    "--workers",
                    "2",
                    "--sparse",
                    "1",
                ],
            )
            .unwrap(),
            release_example_args("gel-phase-quad", "quad_evidence", &["--demo"]).unwrap(),
        ];
        for (package, example, args) in VERIFY_LOOP_EXAMPLES {
            runs.push(release_example_args(package, example, args).unwrap());
        }
        runs
    }

    fn example_of(run: &[&'static str]) -> (&'static str, &'static str) {
        let at = |flag: &str| {
            let i = run.iter().position(|a| *a == flag).unwrap();
            run[i + 1]
        };
        (at("-p"), at("--example"))
    }

    #[test]
    fn example_commands_match_the_reference_literals() {
        assert_eq!(current_runs(), reference_runs());
    }

    #[test]
    fn release_build_is_the_reference_build_plus_every_run_example() {
        let build = verify_release_build_args();
        assert_eq!(&build[..REFERENCE_BUILD.len()], REFERENCE_BUILD);
        let rest = &build[REFERENCE_BUILD.len()..];
        assert_eq!(&rest[..2], ["--lib", "--bins"]);
        let built: Vec<&str> = rest[2..]
            .chunks(2)
            .map(|pair| {
                assert_eq!(pair.len(), 2);
                assert_eq!(pair[0], "--example");
                pair[1]
            })
            .collect();
        let mut ran: Vec<(&str, &str)> = reference_runs().iter().map(|r| example_of(r)).collect();
        assert_eq!(ran.len(), 12);
        ran.sort_unstable();
        ran.dedup();
        assert_eq!(ran.len(), 12, "every example runs once");
        let mut table = VERIFY_RELEASE_EXAMPLES.to_vec();
        table.sort_unstable();
        assert_eq!(
            table, ran,
            "the build table is exactly the set of run examples"
        );
        let mut built_sorted = built.clone();
        built_sorted.sort_unstable();
        let mut names: Vec<&str> = ran.iter().map(|(_, e)| *e).collect();
        names.sort_unstable();
        assert_eq!(built_sorted, names);
        assert_eq!(
            built,
            VERIFY_RELEASE_EXAMPLES
                .iter()
                .map(|(_, e)| *e)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn every_built_example_is_one_unique_workspace_target() {
        let root = workspace_root().unwrap();
        let mut all: Vec<(String, String)> = Vec::new();
        let crates = root.join("crates");
        for krate in fs::read_dir(&crates).unwrap() {
            let krate = krate.unwrap().path();
            let package = krate.file_name().unwrap().to_string_lossy().into_owned();
            let Ok(entries) = fs::read_dir(krate.join("examples")) else {
                continue;
            };
            for entry in entries {
                let path = entry.unwrap().path();
                let name = if path.is_dir() {
                    if !path.join("main.rs").is_file() {
                        continue;
                    }
                    path.file_name().unwrap().to_string_lossy().into_owned()
                } else if path.extension().is_some_and(|x| x == "rs") {
                    path.file_stem().unwrap().to_string_lossy().into_owned()
                } else {
                    continue;
                };
                all.push((package.clone(), name));
            }
        }
        assert!(!root.join("xtask/examples").exists());
        for (package, example) in VERIFY_RELEASE_EXAMPLES {
            let hits: Vec<&(String, String)> = all.iter().filter(|(_, e)| e == example).collect();
            assert_eq!(hits.len(), 1, "{example} must name one workspace example");
            assert_eq!(hits[0].0, *package, "{example} lives in {package}");
        }
    }

    #[test]
    fn seeded_argument_lists_compose_like_the_reference_loop() {
        const POOL: &[&str] = &[
            "--demo",
            "--",
            "-p",
            "--example",
            "",
            "x",
            "--orbs",
            "32",
            "docs/a b",
            "ą",
        ];
        let mut state: u64 = 0x9e37_79b9_7f4a_7c15;
        let mut next = || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        for round in 0..4096 {
            let (package, example) =
                VERIFY_RELEASE_EXAMPLES[(next() as usize) % VERIFY_RELEASE_EXAMPLES.len()];
            let len = if round < 16 {
                round % 2
            } else {
                (next() % 9) as usize
            };
            let args: Vec<&str> = (0..len)
                .map(|_| POOL[(next() as usize) % POOL.len()])
                .collect();
            assert_eq!(
                release_example_args(package, example, &args).unwrap(),
                reference_run(package, example, args.clone())
            );
        }
    }

    #[test]
    fn examples_outside_the_build_are_refused() {
        for (package, example) in [
            ("gel-cli", "data_integrity"),
            ("gel-source", "collection_campaign"),
            ("gel-cli", "precision_matrix"),
            ("gel-reader", "topk_compare"),
            ("gel-cli", "record_history"),
            ("gel-source", "quad_compare"),
            ("", ""),
        ] {
            assert_eq!(
                release_example_args(package, example, &[]),
                Err(format!(
                    "{package} example {example} is not in the verify release build"
                ))
            );
        }
    }
}
