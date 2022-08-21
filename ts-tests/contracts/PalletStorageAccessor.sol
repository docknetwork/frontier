pragma solidity ^0.8.2;

contract PalletStorageAccessor {
  enum KeyType {
    NoKey,
    MapKey,
    DoubleMapKey
  }

  enum Params {
    None,
    Offset,
    Length,
    OffsetAndLength
  }

  function getStorage(string calldata pallet, string calldata member, KeyType keyType, bytes calldata firstKey, bytes calldata secondKey) public returns (bool, bytes memory value) {
    address palletStorageReaderAddress = address(0x0000000000000000000000000000000000000009);
    bytes memory encodedKey = encodeKey(keyType, firstKey, secondKey);
    bytes memory palletBytes = bytes(pallet);
    bytes memory memberBytes = bytes(member);

    return palletStorageReaderAddress.call(
      abi.encodePacked(
        compact(palletBytes.length),
        palletBytes,
        compact(memberBytes.length),
        memberBytes,
        encodedKey,
        Params.None
      )
    );
  }

  function getStorageWithOffset(string calldata pallet, string calldata member, KeyType keyType, bytes calldata firstKey, bytes calldata secondKey, uint32 offset) public returns (bool, bytes memory value) {
    address palletStorageReaderAddress = address(0x0000000000000000000000000000000000000009);
    bytes memory encodedKey = encodeKey(keyType, firstKey, secondKey);
    bytes memory palletBytes = bytes(pallet);
    bytes memory memberBytes = bytes(member);

    return palletStorageReaderAddress.call(
      abi.encodePacked(
        compact(palletBytes.length),
        palletBytes,
        compact(memberBytes.length),
        memberBytes,
        encodedKey,
        Params.Offset,
        reverse(offset)
      )
    );
  }

  function getStorageWithLen(string calldata pallet, string calldata member, KeyType keyType, bytes calldata firstKey, bytes calldata secondKey, uint32 len) public returns (bool, bytes memory value) {
    address palletStorageReaderAddress = address(0x0000000000000000000000000000000000000009);
    bytes memory encodedKey = encodeKey(keyType, firstKey, secondKey);
    bytes memory palletBytes = bytes(pallet);
    bytes memory memberBytes = bytes(member);

    return palletStorageReaderAddress.call(
      abi.encodePacked(
        compact(palletBytes.length),
        palletBytes,
        compact(memberBytes.length),
        memberBytes,
        encodedKey,
        Params.Length,
        reverse(len)
      )
    );
  }

  function getStorageWithOffsetLen(string calldata pallet, string calldata member, KeyType keyType, bytes calldata firstKey, bytes calldata secondKey, uint32 offset, uint32 len) public returns (bool, bytes memory value) {
    address palletStorageReaderAddress = address(0x0000000000000000000000000000000000000009);
    bytes memory encodedKey = encodeKey(keyType, firstKey, secondKey);
    bytes memory palletBytes = bytes(pallet);
    bytes memory memberBytes = bytes(member);

    return palletStorageReaderAddress.call(
      abi.encodePacked(
        compact(palletBytes.length),
        palletBytes,
        compact(memberBytes.length),
        memberBytes,
        encodedKey,
        Params.OffsetAndLength,
        reverse(offset),
        reverse(len)
      )
    );
  }

  function encodeKey(KeyType keyType, bytes calldata firstKey, bytes calldata secondKey) private pure returns (bytes memory value) {
    bytes memory encodedKey;
    if (keyType == KeyType.NoKey) {
      encodedKey = abi.encodePacked(keyType);
      require(firstKey.length == 0, "First key must be empty");
      require(secondKey.length == 0, "Second key must be empty");
    } else if (keyType == KeyType.MapKey) {
      encodedKey = abi.encodePacked(keyType, compact(firstKey.length), firstKey);
      require(secondKey.length == 0, "Second key must be empty");
    } else {
      encodedKey = abi.encodePacked(
        keyType,
        compact(firstKey.length),
        firstKey,
        compact(secondKey.length),
        secondKey
      );
    }

    return encodedKey;
  }

  function reverse(uint32 input) internal pure returns (uint32 v) {
      v = input;

      // swap bytes
      v = ((v & 0xFF00FF00) >> 8) |
          ((v & 0x00FF00FF) << 8);

      // swap 2-byte long pairs
      v = (v >> 16) | (v << 16);
  }

  function compact(uint256 len) private pure returns (bytes memory value) {
    if (len < 64) {
      return abi.encodePacked(uint8(len) << 2);
    } else if (len < 16384) {
      return abi.encodePacked(uint8(len) << 2 | 1, uint8(len >> 8));
    } else {
      revert("Unimplemented");
    }
  }
}