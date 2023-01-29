type SignatureFn = fn(&[u8]) -> Option<&'static str>;

//arrays of functions for file header recognision. Ordered by the first signature byte for fast search 
static SIGS_00: [SignatureFn; 1] = [is_signature_ico];
static SIGS_01: [SignatureFn; 1] = [|_| None];
static SIGS_02: [SignatureFn; 1] = [|_| None];
static SIGS_03: [SignatureFn; 1] = [|_| None];
static SIGS_04: [SignatureFn; 1] = [is_signature_lz4];
static SIGS_05: [SignatureFn; 1] = [|_| None];
static SIGS_06: [SignatureFn; 1] = [|_| None];
static SIGS_07: [SignatureFn; 1] = [|_| None];
static SIGS_08: [SignatureFn; 1] = [|_| None];
static SIGS_09: [SignatureFn; 1] = [|_| None];
static SIGS_0A: [SignatureFn; 1] = [|_| None];
static SIGS_0B: [SignatureFn; 1] = [|_| None];
static SIGS_0C: [SignatureFn; 1] = [|_| None];
static SIGS_0D: [SignatureFn; 1] = [|_| None];
static SIGS_0E: [SignatureFn; 1] = [|_| None];
static SIGS_0F: [SignatureFn; 1] = [|_| None];
static SIGS_10: [SignatureFn; 1] = [|_| None];
static SIGS_11: [SignatureFn; 1] = [|_| None];
static SIGS_12: [SignatureFn; 1] = [|_| None];
static SIGS_13: [SignatureFn; 1] = [|_| None];
static SIGS_14: [SignatureFn; 1] = [|_| None];
static SIGS_15: [SignatureFn; 1] = [|_| None];
static SIGS_16: [SignatureFn; 1] = [|_| None];
static SIGS_17: [SignatureFn; 1] = [|_| None];
static SIGS_18: [SignatureFn; 1] = [|_| None];
static SIGS_19: [SignatureFn; 1] = [|_| None];
static SIGS_1A: [SignatureFn; 1] = [|_| None];
static SIGS_1B: [SignatureFn; 1] = [|_| None];
static SIGS_1C: [SignatureFn; 1] = [|_| None];
static SIGS_1D: [SignatureFn; 1] = [|_| None];
static SIGS_1E: [SignatureFn; 1] = [|_| None];
static SIGS_1F: [SignatureFn; 1] = [is_signature_gzip];
static SIGS_20: [SignatureFn; 1] = [|_| None];
static SIGS_21: [SignatureFn; 1] = [is_signature_deb];
static SIGS_22: [SignatureFn; 1] = [|_| None];
static SIGS_23: [SignatureFn; 1] = [|_| None];
static SIGS_24: [SignatureFn; 1] = [|_| None];
static SIGS_25: [SignatureFn; 1] = [|_| None];
static SIGS_26: [SignatureFn; 1] = [|_| None];
static SIGS_27: [SignatureFn; 1] = [|_| None];
static SIGS_28: [SignatureFn; 1] = [|_| None];
static SIGS_29: [SignatureFn; 1] = [|_| None];
static SIGS_2A: [SignatureFn; 1] = [|_| None];
static SIGS_2B: [SignatureFn; 1] = [|_| None];
static SIGS_2C: [SignatureFn; 1] = [|_| None];
static SIGS_2D: [SignatureFn; 1] = [|_| None];
static SIGS_2E: [SignatureFn; 1] = [|_| None];
static SIGS_2F: [SignatureFn; 1] = [|_| None];
static SIGS_30: [SignatureFn; 1] = [|_| None];
static SIGS_31: [SignatureFn; 1] = [|_| None];
static SIGS_32: [SignatureFn; 1] = [|_| None];
static SIGS_33: [SignatureFn; 1] = [|_| None];
static SIGS_34: [SignatureFn; 1] = [|_| None];
static SIGS_35: [SignatureFn; 1] = [|_| None];
static SIGS_36: [SignatureFn; 1] = [|_| None];
static SIGS_37: [SignatureFn; 2] = [is_signature_7z, is_signature_zpaq];
static SIGS_38: [SignatureFn; 1] = [|_| None];
static SIGS_39: [SignatureFn; 1] = [|_| None];
static SIGS_3A: [SignatureFn; 1] = [|_| None];
static SIGS_3B: [SignatureFn; 1] = [|_| None];
static SIGS_3C: [SignatureFn; 1] = [|_| None];
static SIGS_3D: [SignatureFn; 1] = [|_| None];
static SIGS_3E: [SignatureFn; 1] = [|_| None];
static SIGS_3F: [SignatureFn; 1] = [|_| None];
static SIGS_40: [SignatureFn; 1] = [|_| None];
static SIGS_41: [SignatureFn; 1] = [|_| None];
static SIGS_42: [SignatureFn; 2] = [is_signature_bmp, is_signature_bzip2];
static SIGS_43: [SignatureFn; 1] = [|_| None];
static SIGS_44: [SignatureFn; 1] = [|_| None];
static SIGS_45: [SignatureFn; 1] = [|_| None];
static SIGS_46: [SignatureFn; 1] = [|_| None];
static SIGS_47: [SignatureFn; 1] = [is_signature_gif];
static SIGS_48: [SignatureFn; 1] = [|_| None];
static SIGS_49: [SignatureFn; 1] = [|_| None];
static SIGS_4A: [SignatureFn; 1] = [|_| None];
static SIGS_4B: [SignatureFn; 1] = [|_| None];
static SIGS_4C: [SignatureFn; 1] = [is_signature_lzip];
static SIGS_4D: [SignatureFn; 3] = [is_signature_cab, is_signature_mzpe, is_signature_midi];
static SIGS_4E: [SignatureFn; 1] = [|_| None];
static SIGS_4F: [SignatureFn; 1] = [|_| None];
static SIGS_50: [SignatureFn; 1] = [is_signature_zip];
static SIGS_51: [SignatureFn; 1] = [|_| None];
static SIGS_52: [SignatureFn; 8] = [is_signature_ani, is_signature_avi, is_signature_cdr, is_signature_dls, is_signature_vcd_dat, is_signature_wav, is_signature_webp, is_signature_rar];
static SIGS_53: [SignatureFn; 1] = [|_| None];
static SIGS_54: [SignatureFn; 1] = [|_| None];
static SIGS_55: [SignatureFn; 1] = [|_| None];
static SIGS_56: [SignatureFn; 1] = [|_| None];
static SIGS_57: [SignatureFn; 1] = [|_| None];
static SIGS_58: [SignatureFn; 1] = [|_| None];
static SIGS_59: [SignatureFn; 1] = [|_| None];
static SIGS_5A: [SignatureFn; 1] = [|_| None];
static SIGS_5B: [SignatureFn; 1] = [|_| None];
static SIGS_5C: [SignatureFn; 1] = [|_| None];
static SIGS_5D: [SignatureFn; 1] = [|_| None];
static SIGS_5E: [SignatureFn; 1] = [|_| None];
static SIGS_5F: [SignatureFn; 1] = [|_| None];
static SIGS_60: [SignatureFn; 1] = [|_| None];
static SIGS_61: [SignatureFn; 1] = [|_| None];
static SIGS_62: [SignatureFn; 1] = [|_| None];
static SIGS_63: [SignatureFn; 1] = [|_| None];
static SIGS_64: [SignatureFn; 1] = [|_| None];
static SIGS_65: [SignatureFn; 1] = [|_| None];
static SIGS_66: [SignatureFn; 1] = [|_| None];
static SIGS_67: [SignatureFn; 1] = [|_| None];
static SIGS_68: [SignatureFn; 1] = [|_| None];
static SIGS_69: [SignatureFn; 1] = [|_| None];
static SIGS_6A: [SignatureFn; 1] = [|_| None];
static SIGS_6B: [SignatureFn; 1] = [is_signature_dmg];
static SIGS_6C: [SignatureFn; 1] = [|_| None];
static SIGS_6D: [SignatureFn; 1] = [|_| None];
static SIGS_6E: [SignatureFn; 1] = [|_| None];
static SIGS_6F: [SignatureFn; 1] = [|_| None];
static SIGS_70: [SignatureFn; 1] = [|_| None];
static SIGS_71: [SignatureFn; 1] = [|_| None];
static SIGS_72: [SignatureFn; 1] = [|_| None];
static SIGS_73: [SignatureFn; 1] = [|_| None];
static SIGS_74: [SignatureFn; 1] = [|_| None];
static SIGS_75: [SignatureFn; 1] = [|_| None];
static SIGS_76: [SignatureFn; 1] = [|_| None];
static SIGS_77: [SignatureFn; 1] = [|_| None];
static SIGS_78: [SignatureFn; 1] = [is_signature_xar];
static SIGS_79: [SignatureFn; 1] = [|_| None];
static SIGS_7A: [SignatureFn; 1] = [is_signature_zpaq_blk];
static SIGS_7B: [SignatureFn; 1] = [|_| None];
static SIGS_7C: [SignatureFn; 1] = [|_| None];
static SIGS_7D: [SignatureFn; 1] = [|_| None];
static SIGS_7E: [SignatureFn; 1] = [|_| None];
static SIGS_7F: [SignatureFn; 1] = [is_signature_elf];
static SIGS_80: [SignatureFn; 1] = [|_| None];
static SIGS_81: [SignatureFn; 1] = [|_| None];
static SIGS_82: [SignatureFn; 1] = [|_| None];
static SIGS_83: [SignatureFn; 1] = [|_| None];
static SIGS_84: [SignatureFn; 1] = [|_| None];
static SIGS_85: [SignatureFn; 1] = [|_| None];
static SIGS_86: [SignatureFn; 1] = [|_| None];
static SIGS_87: [SignatureFn; 1] = [|_| None];
static SIGS_88: [SignatureFn; 1] = [|_| None];
static SIGS_89: [SignatureFn; 2] = [is_signature_png, is_signature_lzo];
static SIGS_8A: [SignatureFn; 1] = [|_| None];
static SIGS_8B: [SignatureFn; 1] = [|_| None];
static SIGS_8C: [SignatureFn; 1] = [|_| None];
static SIGS_8D: [SignatureFn; 1] = [|_| None];
static SIGS_8E: [SignatureFn; 1] = [|_| None];
static SIGS_8F: [SignatureFn; 1] = [|_| None];
static SIGS_90: [SignatureFn; 1] = [|_| None];
static SIGS_91: [SignatureFn; 1] = [|_| None];
static SIGS_92: [SignatureFn; 1] = [|_| None];
static SIGS_93: [SignatureFn; 1] = [|_| None];
static SIGS_94: [SignatureFn; 1] = [|_| None];
static SIGS_95: [SignatureFn; 1] = [|_| None];
static SIGS_96: [SignatureFn; 1] = [|_| None];
static SIGS_97: [SignatureFn; 1] = [|_| None];
static SIGS_98: [SignatureFn; 1] = [|_| None];
static SIGS_99: [SignatureFn; 1] = [|_| None];
static SIGS_9A: [SignatureFn; 1] = [|_| None];
static SIGS_9B: [SignatureFn; 1] = [|_| None];
static SIGS_9C: [SignatureFn; 1] = [|_| None];
static SIGS_9D: [SignatureFn; 1] = [|_| None];
static SIGS_9E: [SignatureFn; 1] = [|_| None];
static SIGS_9F: [SignatureFn; 1] = [|_| None];
static SIGS_A0: [SignatureFn; 1] = [|_| None];
static SIGS_A1: [SignatureFn; 1] = [|_| None];
static SIGS_A2: [SignatureFn; 1] = [|_| None];
static SIGS_A3: [SignatureFn; 1] = [|_| None];
static SIGS_A4: [SignatureFn; 1] = [|_| None];
static SIGS_A5: [SignatureFn; 1] = [|_| None];
static SIGS_A6: [SignatureFn; 1] = [|_| None];
static SIGS_A7: [SignatureFn; 1] = [|_| None];
static SIGS_A8: [SignatureFn; 1] = [|_| None];
static SIGS_A9: [SignatureFn; 1] = [|_| None];
static SIGS_AA: [SignatureFn; 1] = [|_| None];
static SIGS_AB: [SignatureFn; 1] = [|_| None];
static SIGS_AC: [SignatureFn; 1] = [|_| None];
static SIGS_AD: [SignatureFn; 1] = [|_| None];
static SIGS_AE: [SignatureFn; 1] = [|_| None];
static SIGS_AF: [SignatureFn; 1] = [|_| None];
static SIGS_B0: [SignatureFn; 1] = [|_| None];
static SIGS_B1: [SignatureFn; 1] = [|_| None];
static SIGS_B2: [SignatureFn; 1] = [|_| None];
static SIGS_B3: [SignatureFn; 1] = [|_| None];
static SIGS_B4: [SignatureFn; 1] = [|_| None];
static SIGS_B5: [SignatureFn; 1] = [|_| None];
static SIGS_B6: [SignatureFn; 1] = [|_| None];
static SIGS_B7: [SignatureFn; 1] = [|_| None];
static SIGS_B8: [SignatureFn; 1] = [|_| None];
static SIGS_B9: [SignatureFn; 1] = [|_| None];
static SIGS_BA: [SignatureFn; 1] = [|_| None];
static SIGS_BB: [SignatureFn; 1] = [|_| None];
static SIGS_BC: [SignatureFn; 1] = [|_| None];
static SIGS_BD: [SignatureFn; 1] = [|_| None];
static SIGS_BE: [SignatureFn; 1] = [|_| None];
static SIGS_BF: [SignatureFn; 1] = [|_| None];
static SIGS_C0: [SignatureFn; 1] = [|_| None];
static SIGS_C1: [SignatureFn; 1] = [|_| None];
static SIGS_C2: [SignatureFn; 1] = [|_| None];
static SIGS_C3: [SignatureFn; 1] = [|_| None];
static SIGS_C4: [SignatureFn; 1] = [|_| None];
static SIGS_C5: [SignatureFn; 1] = [|_| None];
static SIGS_C6: [SignatureFn; 1] = [|_| None];
static SIGS_C7: [SignatureFn; 1] = [|_| None];
static SIGS_C8: [SignatureFn; 1] = [|_| None];
static SIGS_C9: [SignatureFn; 1] = [|_| None];
static SIGS_CA: [SignatureFn; 1] = [is_signature_java];
static SIGS_CB: [SignatureFn; 1] = [|_| None];
static SIGS_CC: [SignatureFn; 1] = [|_| None];
static SIGS_CD: [SignatureFn; 1] = [|_| None];
static SIGS_CE: [SignatureFn; 1] = [is_signature_macho];
static SIGS_CF: [SignatureFn; 1] = [is_signature_macho];
static SIGS_D0: [SignatureFn; 1] = [|_| None];
static SIGS_D1: [SignatureFn; 1] = [|_| None];
static SIGS_D2: [SignatureFn; 1] = [|_| None];
static SIGS_D3: [SignatureFn; 1] = [|_| None];
static SIGS_D4: [SignatureFn; 1] = [|_| None];
static SIGS_D5: [SignatureFn; 1] = [|_| None];
static SIGS_D6: [SignatureFn; 1] = [|_| None];
static SIGS_D7: [SignatureFn; 1] = [|_| None];
static SIGS_D8: [SignatureFn; 1] = [|_| None];
static SIGS_D9: [SignatureFn; 1] = [|_| None];
static SIGS_DA: [SignatureFn; 1] = [|_| None];
static SIGS_DB: [SignatureFn; 1] = [|_| None];
static SIGS_DC: [SignatureFn; 1] = [|_| None];
static SIGS_DD: [SignatureFn; 1] = [|_| None];
static SIGS_DE: [SignatureFn; 1] = [|_| None];
static SIGS_DF: [SignatureFn; 1] = [|_| None];
static SIGS_E0: [SignatureFn; 1] = [|_| None];
static SIGS_E1: [SignatureFn; 1] = [|_| None];
static SIGS_E2: [SignatureFn; 1] = [|_| None];
static SIGS_E3: [SignatureFn; 1] = [|_| None];
static SIGS_E4: [SignatureFn; 1] = [|_| None];
static SIGS_E5: [SignatureFn; 1] = [|_| None];
static SIGS_E6: [SignatureFn; 1] = [|_| None];
static SIGS_E7: [SignatureFn; 1] = [|_| None];
static SIGS_E8: [SignatureFn; 1] = [|_| None];
static SIGS_E9: [SignatureFn; 1] = [|_| None];
static SIGS_EA: [SignatureFn; 1] = [|_| None];
static SIGS_EB: [SignatureFn; 1] = [|_| None];
static SIGS_EC: [SignatureFn; 1] = [|_| None];
static SIGS_ED: [SignatureFn; 1] = [is_signature_rpm];
static SIGS_EE: [SignatureFn; 1] = [|_| None];
static SIGS_EF: [SignatureFn; 1] = [is_signature_nsis];
static SIGS_F0: [SignatureFn; 1] = [|_| None];
static SIGS_F1: [SignatureFn; 1] = [|_| None];
static SIGS_F2: [SignatureFn; 1] = [|_| None];
static SIGS_F3: [SignatureFn; 1] = [|_| None];
static SIGS_F4: [SignatureFn; 1] = [|_| None];
static SIGS_F5: [SignatureFn; 1] = [|_| None];
static SIGS_F6: [SignatureFn; 1] = [|_| None];
static SIGS_F7: [SignatureFn; 1] = [|_| None];
static SIGS_F8: [SignatureFn; 1] = [|_| None];
static SIGS_F9: [SignatureFn; 1] = [|_| None];
static SIGS_FA: [SignatureFn; 1] = [|_| None];
static SIGS_FB: [SignatureFn; 1] = [|_| None];
static SIGS_FC: [SignatureFn; 1] = [|_| None];
static SIGS_FD: [SignatureFn; 1] = [is_signature_xz];
static SIGS_FE: [SignatureFn; 1] = [|_| None];
static SIGS_FF: [SignatureFn; 1] = [is_signature_jpeg];

//map of all sig fn indexed by first byte
static SIG_MAP: [&[SignatureFn]; 256] = [
                &SIGS_00, &SIGS_01, &SIGS_02, &SIGS_03, &SIGS_04, &SIGS_05, &SIGS_06, &SIGS_07, &SIGS_08, &SIGS_09, &SIGS_0A, &SIGS_0B, &SIGS_0C, &SIGS_0D, &SIGS_0E, &SIGS_0F,
                &SIGS_10, &SIGS_11, &SIGS_12, &SIGS_13, &SIGS_14, &SIGS_15, &SIGS_16, &SIGS_17, &SIGS_18, &SIGS_19, &SIGS_1A, &SIGS_1B, &SIGS_1C, &SIGS_1D, &SIGS_1E, &SIGS_1F,
                &SIGS_20, &SIGS_21, &SIGS_22, &SIGS_23, &SIGS_24, &SIGS_25, &SIGS_26, &SIGS_27, &SIGS_28, &SIGS_29, &SIGS_2A, &SIGS_2B, &SIGS_2C, &SIGS_2D, &SIGS_2E, &SIGS_2F,
                &SIGS_30, &SIGS_31, &SIGS_32, &SIGS_33, &SIGS_34, &SIGS_35, &SIGS_36, &SIGS_37, &SIGS_38, &SIGS_39, &SIGS_3A, &SIGS_3B, &SIGS_3C, &SIGS_3D, &SIGS_3E, &SIGS_3F,
                &SIGS_40, &SIGS_41, &SIGS_42, &SIGS_43, &SIGS_44, &SIGS_45, &SIGS_46, &SIGS_47, &SIGS_48, &SIGS_49, &SIGS_4A, &SIGS_4B, &SIGS_4C, &SIGS_4D, &SIGS_4E, &SIGS_4F,
                &SIGS_50, &SIGS_51, &SIGS_52, &SIGS_53, &SIGS_54, &SIGS_55, &SIGS_56, &SIGS_57, &SIGS_58, &SIGS_59, &SIGS_5A, &SIGS_5B, &SIGS_5C, &SIGS_5D, &SIGS_5E, &SIGS_5F,
                &SIGS_60, &SIGS_61, &SIGS_62, &SIGS_63, &SIGS_64, &SIGS_65, &SIGS_66, &SIGS_67, &SIGS_68, &SIGS_69, &SIGS_6A, &SIGS_6B, &SIGS_6C, &SIGS_6D, &SIGS_6E, &SIGS_6F,
                &SIGS_70, &SIGS_71, &SIGS_72, &SIGS_73, &SIGS_74, &SIGS_75, &SIGS_76, &SIGS_77, &SIGS_78, &SIGS_79, &SIGS_7A, &SIGS_7B, &SIGS_7C, &SIGS_7D, &SIGS_7E, &SIGS_7F,
                &SIGS_80, &SIGS_81, &SIGS_82, &SIGS_83, &SIGS_84, &SIGS_85, &SIGS_86, &SIGS_87, &SIGS_88, &SIGS_89, &SIGS_8A, &SIGS_8B, &SIGS_8C, &SIGS_8D, &SIGS_8E, &SIGS_8F,
                &SIGS_90, &SIGS_91, &SIGS_92, &SIGS_93, &SIGS_94, &SIGS_95, &SIGS_96, &SIGS_97, &SIGS_98, &SIGS_99, &SIGS_9A, &SIGS_9B, &SIGS_9C, &SIGS_9D, &SIGS_9E, &SIGS_9F,
                &SIGS_A0, &SIGS_A1, &SIGS_A2, &SIGS_A3, &SIGS_A4, &SIGS_A5, &SIGS_A6, &SIGS_A7, &SIGS_A8, &SIGS_A9, &SIGS_AA, &SIGS_AB, &SIGS_AC, &SIGS_AD, &SIGS_AE, &SIGS_AF,
                &SIGS_B0, &SIGS_B1, &SIGS_B2, &SIGS_B3, &SIGS_B4, &SIGS_B5, &SIGS_B6, &SIGS_B7, &SIGS_B8, &SIGS_B9, &SIGS_BA, &SIGS_BB, &SIGS_BC, &SIGS_BD, &SIGS_BE, &SIGS_BF,
                &SIGS_C0, &SIGS_C1, &SIGS_C2, &SIGS_C3, &SIGS_C4, &SIGS_C5, &SIGS_C6, &SIGS_C7, &SIGS_C8, &SIGS_C9, &SIGS_CA, &SIGS_CB, &SIGS_CC, &SIGS_CD, &SIGS_CE, &SIGS_CF,
                &SIGS_D0, &SIGS_D1, &SIGS_D2, &SIGS_D3, &SIGS_D4, &SIGS_D5, &SIGS_D6, &SIGS_D7, &SIGS_D8, &SIGS_D9, &SIGS_DA, &SIGS_DB, &SIGS_DC, &SIGS_DD, &SIGS_DE, &SIGS_DF,
                &SIGS_E0, &SIGS_E1, &SIGS_E2, &SIGS_E3, &SIGS_E4, &SIGS_E5, &SIGS_E6, &SIGS_E7, &SIGS_E8, &SIGS_E9, &SIGS_EA, &SIGS_EB, &SIGS_EC, &SIGS_ED, &SIGS_EE, &SIGS_EF,
                &SIGS_F0, &SIGS_F1, &SIGS_F2, &SIGS_F3, &SIGS_F4, &SIGS_F5, &SIGS_F6, &SIGS_F7, &SIGS_F8, &SIGS_F9, &SIGS_FA, &SIGS_FB, &SIGS_FC, &SIGS_FD, &SIGS_FE, &SIGS_FF
                ];

pub fn get_signature(data: &[u8]) -> Option<&'static str> {

    if let Some(&sig_idx) = data.first() {
        return SIG_MAP[sig_idx as usize]
            .iter()
            .find_map(|f| f(data));
    }
    None
}

//try to recognize DIB header. 
fn is_signature_dib(data: &[u8]) -> bool {
    
    if data.len() > 15 && data.starts_with(&[40, 0, 0, 0]) {
        return data[12] == 1 && data[13] == 0 && (data[14] == 1 || data[14] == 4 || data[14] == 8 || data[14] == 24 || data[14] == 32) && data[15] == 0;
    }
    false
}

//try to recognize BMP header
fn is_signature_bmp(data: &[u8]) -> Option<&'static str> {

    if data.len() > 54 && data[0] == 0x42 && data[1] == 0x4D && data[6..10].starts_with(&[0, 0, 0, 0]) {

        //following 4 bytes are address of picture data. Should not be less then 0x36 and also not too much
        let pic_offset = u32::from_le_bytes(data[10..14].try_into().unwrap()) as usize;
        return ((0x36..=0xFFFF).contains(&pic_offset) && is_signature_dib(&data[14..])).then_some("bmp");
    }
    None
}

//try to recognize PNG header
fn is_signature_png(data: &[u8]) -> Option<&'static str> {

    if data.len() > 16 && data.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {

        //next must be IHDR chunk. Is always 13 bytes long (big endian)
        return data[8..16].starts_with(&[0, 0, 0, 0x0D, 0x49, 0x48, 0x44, 0x52]).then_some("png");
    }
    None
}

//try to recognize ICO/CUR header
fn is_signature_ico(data: &[u8]) -> Option<&'static str> {

    //ico starts with a lot of non uniqe bytes
    if data.len() > 54 && data[0] == 0 && data[1] == 0 && (data[2] == 1 || data[2] == 2) && data[3] == 0 && data[5] == 0 && data[9] == 0 {

        let image_offset = u32::from_le_bytes(data[18..22].try_into().unwrap()) as usize;
        if image_offset >= 22 && image_offset < data.len() {

            //need to check if it points to PNG or DIB struct
            return (is_signature_png(&data[image_offset..]).is_some() || is_signature_dib(&data[image_offset..])).then_some("ico");
        }
    }
    None
}

//try to recognize ani header
fn is_signature_ani(data: &[u8]) -> Option<&'static str> {

    //check for RIFF + ACON magic
    (data.len() > 16 && data.starts_with(&[0x52, 0x49, 0x46, 0x46]) && data[8..12].starts_with(&[0x41, 0x43, 0x4F, 0x4E])).then_some("ani")
}

//try to recognize cdr header
fn is_signature_cdr(data: &[u8]) -> Option<&'static str> {

    //check for RIFF + CDR magic
    (data.len() > 16 && data.starts_with(&[0x52, 0x49, 0x46, 0x46]) && data[8..12].starts_with(&[0x43, 0x44, 0x52])).then_some("cdr")
}

//try to recognize GIF header
fn is_signature_gif(data: &[u8]) -> Option<&'static str> {

    //check magic GIF89a
    if data.len() > 11 && data.starts_with(&[0x47,0x49,0x46,0x38,0x39,0x61]) {

        //find start of extension block. Should starts with '!'
        let offset = (1 << ((data[10] & 0x07) + 1) as usize) * 3 + 13;
        return (data.len() > offset && data[offset] == 0x21).then_some("gif");
    }
    None
}

//try to recognize JPEG header
fn is_signature_jpeg(data: &[u8]) -> Option<&'static str> {

    if data.len() > 12 && data.starts_with(&[0xFF, 0xD8, 0xFF]) { 

        //following with APP0 or APP1 segment with JFIF\x00 or Exif\x00 bytes
        if data[3] == 0xE0 {
            return data[6..11].starts_with(&[0x4A, 0x46, 0x49, 0x46, 0]).then_some("jpeg");

        } else if data[3] == 0xE1 {
            return data[6..11].starts_with(&[0x45, 0x78, 0x69, 0x66, 0]).then_some("jpeg");
        }
    }
    None
}

//try to recognize webp header
fn is_signature_webp(data: &[u8]) -> Option<&'static str> {

    //check for RIFF + WEBP magic
    (data.len() > 16 && data.starts_with(&[0x52, 0x49, 0x46, 0x46]) && data[8..12].starts_with(&[0x57, 0x45, 0x42, 0x50])).then_some("webp")
}

//try to recognize ZIP header
fn is_signature_zip(data: &[u8]) -> Option<&'static str> {

    //check for PK magic
    if data.len() > 8 && data[0] == 0x50 && data[1] == 0x4B {

        //check for various block types
        let d2 = data[2];
        let d3 = data[3];
        return ((d2 == 3 && d3 == 4) || (d2 == 6 && d3 == 8) || (d2 == 1 && d3 == 2) 
                || (d2 == 6 && d3 == 6) || (d2 == 6 && d3 == 7) || (d2 == 5 && d3 == 6) || (d2 == 5 && d3 == 5)).then_some("zip");
    }
    None
}

//try to recognize RAR header
fn is_signature_rar(data: &[u8]) -> Option<&'static str> {

    (data.len() > 8 && data.starts_with(&[0x52, 0x61, 0x72, 0x21, 0x1A, 0x07]) && (data[6] == 0x01 || data[6] == 0x00)).then_some("rar")
}

//try to recognize 7z header
fn is_signature_7z(data: &[u8]) -> Option<&'static str> {

    (data.len() > 8 && data.starts_with(&[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C])).then_some("7zip")
}

//try to recognize xz header
fn is_signature_xz(data: &[u8]) -> Option<&'static str> {

    (data.len() > 8 && data.starts_with(&[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0, 0])).then_some("xz")
}

//try to recognize bzip2 header
fn is_signature_bzip2(data: &[u8]) -> Option<&'static str> {

    (data.len() > 12 && data[0] == 0x42 && data[1] == 0x5A && data[2] == 0x68 && data[3] >= 0x31 && data[3] <= 0x39 
            && data[4..10].starts_with(&[0x31, 0x41, 0x59, 0x26, 0x53, 0x59])).then_some("bzip2")
}

//try to recognize gz header
fn is_signature_gzip(data: &[u8]) -> Option<&'static str> {

    (data.len() > 12 && data[0] == 0x1F && data[1] == 0x8B && data[2] == 0x08 && data[3] <= 0x1F 
            && (data[8] == 0 || data[8] == 2 || data[8] == 4) && (data[9] <= 13 || data[9] == 0xFF)).then_some("gzip")
}

//try to recognize lzip header
fn is_signature_lzip(data: &[u8]) -> Option<&'static str> {

    (data.len() > 12 && data.starts_with(&[0x4C, 0x5A, 0x49, 0x50, 0x01])).then_some("lzip")
}

//try to recognize lzo header
fn is_signature_lzo(data: &[u8]) -> Option<&'static str> {

    (data.len() > 12 && data.starts_with(&[0x89, 0x4c, 0x5a, 0x4f, 0x00, 0x0d, 0x0a, 0x1a, 0x0a])).then_some("lzo")
}

//try to recognize cab header
fn is_signature_cab(data: &[u8]) -> Option<&'static str> {

    (data.len() > 26 && data[0] == 0x4D && data[1] == 0x53 && data[2] == 0x43 && data[3] <= 0x46 
            && data[4] == 0 && data[5] == 0 && data[6] == 0 && data[7] == 0 
            && data[24] == 0x03 && data[25] == 0x01).then_some("cab")
}

//try to recognize zpaq header
fn is_signature_zpaq(data: &[u8]) -> Option<&'static str> {

    (data.len() > 13 && data.starts_with(&[0x37, 0x6B, 0x53, 0x74, 0xA0, 0x31, 0x83, 0xD3, 0x8C, 0xB2, 0x28, 0xB0, 0xD3])).then_some("zpaq")
}

//try to recognize zpaq block header
fn is_signature_zpaq_blk(data: &[u8]) -> Option<&'static str> {

    (data.len() > 5 && data[0] == 0x7A && data[1] == 0x50 && data[2] == 0x51 && (data[3] == 2 || data[3] == 1) && data[4] == 1).then_some("zpaq_blk")
}

//try to recognize xar header
fn is_signature_xar(data: &[u8]) -> Option<&'static str> {

    (data.len() > 28 && data.starts_with(&[0x78, 0x61, 0x72, 0x21, 0x00, 0x1C, 0x00, 0x01])).then_some("xar")
}

//try to recognize lz4 header
fn is_signature_lz4(data: &[u8]) -> Option<&'static str> {

    (data.len() > 12 && data.starts_with(&[0x04, 0x22, 0x4D, 0x18]) && data[4] & 0b11000010 == 0b01000000).then_some("lz4")
}

//try to recognize nsis header
fn is_signature_nsis(data: &[u8]) -> Option<&'static str> {

    (data.len() > 16 && data.starts_with(&[0xEF, 0xBE, 0xAD, 0xDE, 0x4E, 0x75, 0x6C, 0x6C, 0x73, 0x6F, 0x66, 0x74, 0x49, 0x6E, 0x73, 0x74])).then_some("nsis")
}

//try to recognize java header
fn is_signature_java(data: &[u8]) -> Option<&'static str> {

    if data.len() > 16 && data.starts_with(&[0xCA, 0xFE, 0xBA, 0xBE]) {
        return Some(if data[6] == 0 && data[7] >= 0x2D {
            "java"
        } else {
            "fat"
        });
    }
    None
}

//try to recognize dmg header
fn is_signature_dmg(data: &[u8]) -> Option<&'static str> {

    (data.len() > 12 && data.starts_with(&[0x6B, 0x6F, 0x6C, 0x79, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x02, 0x00])).then_some("dmg")
}

//try to recognize deb header
fn is_signature_deb(data: &[u8]) -> Option<&'static str> {

    (data.len() > 0x26 && data.starts_with(&[0x21, 0x3C, 0x61, 0x72, 0x63, 0x68, 0x3E, 0x0A, 0x64, 0x65, 0x62, 0x69, 0x61, 0x6E, 0x2D, 0x62, 0x69, 0x6E, 0x61, 0x72, 0x79])
            && data[66..68].starts_with(&[0x60, 0x0A])).then_some("deb")
}

//try to recognize rpm header
fn is_signature_rpm(data: &[u8]) -> Option<&'static str> {

    (data.len() > 0x70 && data.starts_with(&[0xED, 0xAB, 0xEE, 0xDB, 0x03, 0]) && data[0x60..0x63].starts_with(&[0x8E, 0xAD, 0xE8])).then_some("rpm")
}

//try to recognize exe/mzpe header
fn is_signature_mzpe(data: &[u8]) -> Option<&'static str> {

    if data.len() > 0x40 && data[0] == 0x4D && data[1] == 0x5A { 

        //dword at 0x3C offset should point to PE\x00\x00
        let pe_offset = u32::from_le_bytes(data[0x3C..0x40].try_into().unwrap()) as usize;
        return (pe_offset < data.len() && data[pe_offset] == 0x50 && data[pe_offset + 1] == 0x45 && data[pe_offset + 2] == 0 && data[pe_offset + 3] == 0).then_some("mzpe");
    }
    None
}

//try to recognize elf header
fn is_signature_elf(data: &[u8]) -> Option<&'static str> {

    if data.len() > 8 && data.starts_with(&[0x7F, 0x45, 0x4C, 0x46]) {

        //addition check for 32/64 bit flag, endianness and version
        return ((data[4] == 1 || data[4] == 2) && (data[5] == 1 || data[5] == 2) && data[6] == 1).then_some("elf");
    }
    None
}

//try to recognize macho header
//TODO: add more recognision cputype filetype .. etc
fn is_signature_macho(data: &[u8]) -> Option<&'static str> {

    (data.len() > 12 && (data.starts_with(&[0xCE, 0xFA, 0xED, 0xFE]) ||
            //data.starts_with(&[0xFE, 0xED, 0xFA, 0xCE]) ||
            data.starts_with(&[0xCF, 0xFA, 0xED, 0xFE]))).then_some("mach-o")
            //data.starts_with(&[0xFE, 0xED, 0xFA, 0xCF]))).then_some("mach-o")
}

//try to recognize DLS header
fn is_signature_dls(data: &[u8]) -> Option<&'static str> {

    //check for RIFF + DLS magic
    (data.len() > 16 && data.starts_with(&[0x52, 0x49, 0x46, 0x46]) && data[8..12].starts_with(&[0x44, 0x4C, 0x53, 0x20])).then_some("dls")
}

//try to recognize wav header
fn is_signature_wav(data: &[u8]) -> Option<&'static str> {

    (data.len() > 16 && data.starts_with(&[0x52, 0x49, 0x46, 0x46]) && data[8..12].starts_with(&[0x57, 0x41, 0x56, 0x45])).then_some("wav")
}

//try to recognize midi header
fn is_signature_midi(data: &[u8]) -> Option<&'static str> {

    (data.len() > 18 && data.starts_with(&[0x4D, 0x54, 0x68, 0x64, 0, 0, 0, 0x06]) && data[14..18].starts_with(&[0x4D, 0x54, 0x72, 0x6B])).then_some("midi")
}

//try to recognize avi header
fn is_signature_avi(data: &[u8]) -> Option<&'static str> {

    //check for RIFF + AVI magic
    (data.len() > 16 && data.starts_with(&[0x52, 0x49, 0x46, 0x46]) && data[8..12].starts_with(&[0x41, 0x56, 0x49, 0x20])).then_some("avi")
}

//try to recognize vcd_dat header
fn is_signature_vcd_dat(data: &[u8]) -> Option<&'static str> {

    //check for RIFF + CDXA magic
    (data.len() > 16 && data.starts_with(&[0x52, 0x49, 0x46, 0x46]) && data[8..12].starts_with(&[0x43, 0x44, 0x58, 0x41])).then_some("vcd_dat")
}
