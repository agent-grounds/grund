class Block {
    string raw = """"
        a """ run of three does not close a run of four
        see §api/FS-gone-csharp-raw and §FS-gone-csharp-raw
        """";
    string verbatim = @"
        a doubled "" quote does not close it
        see §api/FS-gone-csharp-verbatim and §FS-gone-csharp-verbatim
        ";
}
