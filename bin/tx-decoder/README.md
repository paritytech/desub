# Tx Decoder
### Tool that decodes transactions in a given SQL database


The goal of TxDecoder is to be a handy CLI tool for decoding extrinsics/storage that exist in an archive SQL database.
- Decode all extrinsics of a certain spec (one or multiple or every), and return all those that were not succesful
- Decode a specific extrinsic based on block_num & optional extrinsic index (no index will decode all ext).
- Decode storage entries of a specific prefix of all blocks in a specific spec version/time period/etc
- Decode specific storage entry of a specific block
- Decode all storage entries of a specific block


