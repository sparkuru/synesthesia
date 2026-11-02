$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

function Skip-Bytes([System.IO.BinaryReader] $reader, [UInt64] $count) {
    $remaining = $reader.BaseStream.Length - $reader.BaseStream.Position
    if ($count -gt [UInt64]$remaining) { throw 'Truncated GGUF metadata.' }
    [void]$reader.BaseStream.Seek([Int64]$count, [System.IO.SeekOrigin]::Current)
}

function Read-String([System.IO.BinaryReader] $reader) {
    $length = $reader.ReadUInt64()
    if ($length -gt 1048576) { throw 'GGUF metadata string exceeds inspection limit.' }
    if ($length -gt ($reader.BaseStream.Length - $reader.BaseStream.Position)) { throw 'Truncated GGUF string.' }
    return [System.Text.Encoding]::UTF8.GetString($reader.ReadBytes([Int32]$length))
}

function Skip-Value([System.IO.BinaryReader] $reader, [UInt32] $type, [Int32] $depth) {
    if ($depth -gt 4) { throw 'Nested GGUF array exceeds inspection limit.' }
    switch ($type) {
        { $_ -in 0, 1, 7 } { Skip-Bytes $reader 1; return }
        { $_ -in 2, 3 } { Skip-Bytes $reader 2; return }
        { $_ -in 4, 5, 6 } { Skip-Bytes $reader 4; return }
        { $_ -in 10, 11, 12 } { Skip-Bytes $reader 8; return }
        8 { Skip-Bytes $reader ($reader.ReadUInt64()); return }
        9 {
            $item_type = $reader.ReadUInt32()
            $count = $reader.ReadUInt64()
            if ($count -gt 1000000) { throw 'GGUF array exceeds inspection limit.' }
            $width = switch ($item_type) { { $_ -in 0, 1, 7 } { 1 }; { $_ -in 2, 3 } { 2 }; { $_ -in 4, 5, 6 } { 4 }; { $_ -in 10, 11, 12 } { 8 }; default { 0 } }
            if ($width -gt 0) { Skip-Bytes $reader ($count * $width); return }
            for ($index = 0; $index -lt $count; $index++) { Skip-Value $reader $item_type ($depth + 1) }
            return
        }
        default { throw 'Unknown GGUF metadata type.' }
    }
}

function Read-Scalar([System.IO.BinaryReader] $reader, [UInt32] $type) {
    switch ($type) {
        0 { return $reader.ReadByte() }
        1 { return $reader.ReadSByte() }
        2 { return $reader.ReadUInt16() }
        3 { return $reader.ReadInt16() }
        4 { return $reader.ReadUInt32() }
        5 { return $reader.ReadInt32() }
        6 { return $reader.ReadSingle() }
        7 { return $reader.ReadBoolean() }
        8 { return Read-String $reader }
        10 { return $reader.ReadUInt64() }
        11 { return $reader.ReadInt64() }
        12 { return $reader.ReadDouble() }
        default { throw 'Expected scalar GGUF metadata.' }
    }
}

function Inspect-Model([String] $path) {
    $stream = [System.IO.File]::Open($path, [System.IO.FileMode]::Open, [System.IO.FileAccess]::Read, [System.IO.FileShare]::Read)
    $reader = [System.IO.BinaryReader]::new($stream)
    try {
        if ($reader.ReadUInt32() -ne 0x46554747) { throw 'Invalid GGUF magic.' }
        $version = $reader.ReadUInt32()
        if ($version -notin 2, 3) { throw 'Unsupported GGUF inspection version.' }
        $tensors = $reader.ReadUInt64()
        if ($tensors -gt 100000) { throw 'GGUF tensor count exceeds inspection limit.' }
        $entries = $reader.ReadUInt64()
        if ($entries -gt 100000) { throw 'GGUF metadata entries exceed inspection limit.' }
        $values = [ordered]@{ file = [System.IO.Path]::GetFileName($path); gguf_version = $version; tensor_count = $tensors; metadata_count = $entries }
        for ($index = 0; $index -lt $entries; $index++) {
            $key = Read-String $reader
            $type = $reader.ReadUInt32()
            $selected = $key -in 'general.architecture', 'general.name', 'general.file_type', 'general.quantization_version', 'tokenizer.ggml.model', 'tokenizer.ggml.bos_token_id', 'tokenizer.ggml.eos_token_id', 'tokenizer.ggml.add_bos_token', 'tokenizer.ggml.add_eos_token'
            $selected = $selected -or $key -match '\.(context_length|embedding_length|block_count)$'
            if ($selected -and $type -ne 9) { $values[$key] = Read-Scalar $reader $type; continue }
            if ($key -eq 'tokenizer.chat_template' -and $type -eq 8) {
                $template = Read-String $reader
                $hash = [System.Security.Cryptography.SHA256]::Create()
                try { $values['chat_template_sha256'] = ([BitConverter]::ToString($hash.ComputeHash([System.Text.Encoding]::UTF8.GetBytes($template)))).Replace('-', '').ToLowerInvariant() }
                finally { $hash.Dispose() }
                $values['chat_template_utf8_bytes'] = [System.Text.Encoding]::UTF8.GetByteCount($template)
                continue
            }
            Skip-Value $reader $type 0
        }
        $tensor_types = [ordered]@{}
        for ($index = 0; $index -lt $tensors; $index++) {
            [void](Read-String $reader)
            $dimensions = $reader.ReadUInt32()
            if ($dimensions -lt 1 -or $dimensions -gt 4) { throw 'Invalid GGUF tensor dimensions.' }
            Skip-Bytes $reader ($dimensions * 8)
            $tensor_type = [String]$reader.ReadUInt32()
            Skip-Bytes $reader 8
            if (-not $tensor_types.Contains($tensor_type)) { $tensor_types[$tensor_type] = 0 }
            $tensor_types[$tensor_type]++
        }
        $values['tensor_type_counts'] = $tensor_types
        return $values
    }
    finally { $reader.Dispose(); $stream.Dispose() }
}

@(
    Inspect-Model 'E:\tmp\models\qwen3-asr-1.7b-q4_k.gguf'
    Inspect-Model 'E:\tmp\models\Hy-MT2-1.8B-UD-Q3_K_XL.gguf'
) | ConvertTo-Json -Depth 4
