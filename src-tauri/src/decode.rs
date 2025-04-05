use hyper::body::Bytes;

pub fn initialize(buffer: Bytes, key: String, offset: Option<i64>) -> Bytes {
    let decrypted_key = decrypt_partial_key(buffer);
    let res = initialize_decryptor(ctx, decryptedPartialKeyBuf, decryptedPartialKeyLen);
 
    match offset {
        Some(value) => consume(res, offset),
        None => (),
    }

    return res;
}

pub fn partial_key() -> (Bytes, Bytes) {
    let mut mask = 0xbb;
    let decode_str = "PartialKeyEncKey".as_bytes();
    let decode_melon = "!@melon!@melon!@".as_bytes();
    let decode_char = [
        10, 99, 3, 101, 89, 28, 115, 21, 101, 13, 65, 67, 98, 90, 95, 65,
    ];

    let mut key_buf = [0; 16];
    let mut iv_buf = [0; 16];
    let mut buf = 0;

    for i in 0..decode_str.len() {
        key_buf[i] = decode_str[i] ^ mask ^ decode_melon[i];
        mask = mask - 1;
    }

    for i in 0..decode_char.len() {
        buf = decode_char[i] ^ buf & 0xFF;
        iv_buf[i] = buf;
    }

    for i in 0..key_buf.len() {
        key_buf[i] ^= buf & 0xFF;
        iv_buf[i] ^= buf & 0xFF;
    }

    return (key_buf, iv_buf);
}

pub fn decrypt_partial_key(buffer: Bytes) -> String {
    let (key_buf, iv_buf) = partial_key();

    let key = aes_set_decrypt_key_impl(key_buf);

    let cipher = base64_decode_ctx(buffer);
    crypto_cbc128_decrypt(cipher, out, cipherLen, &key, ivBuf);
  
    // free(cipher);
    let plainLen = cipherLen - out[cipherLen-1];

    return plainLen;
}

pub fn crypto_cbc128_decrypt(in, out, len, key, ivec, block) {
    size_t n;
    union {
        size_t t[16 / sizeof(size_t)];
        unsigned char c[16];
    } tmp;

    #if !defined(OPENSSL_SMALL_FOOTPRINT)
    if (in != out) {
        const unsigned char *iv = ivec;

        if (STRICT_ALIGNMENT &&
                ((size_t)in | (size_t)out | (size_t)ivec) % sizeof(size_t) != 0) {
            while (len >= 16) {
                (*block) (in, out, key);
                for (n = 0; n < 16; ++n)
                    out[n] ^= iv[n];
                iv = in;
                len -= 16;
                in += 16;
                out += 16;
            }
        } else if (16 % sizeof(size_t) == 0) { /* always true */
            while (len >= 16) {
                size_t *out_t = (size_t *)out, *iv_t = (size_t *)iv;

                (*block) (in, out, key);
                for (n = 0; n < 16 / sizeof(size_t); n++)
                    out_t[n] ^= iv_t[n];
                iv = in;
                len -= 16;
                in += 16;
                out += 16;
            }
        }
        memcpy(ivec, iv, 16);
    } else {
        if (STRICT_ALIGNMENT &&
                ((size_t)in | (size_t)out | (size_t)ivec) % sizeof(size_t) != 0) {
            unsigned char c;
            while (len >= 16) {
                (*block) (in, tmp.c, key);
                for (n = 0; n < 16; ++n) {
                    c = in[n];
                    out[n] = tmp.c[n] ^ ivec[n];
                    ivec[n] = c;
                }
                len -= 16;
                in += 16;
                out += 16;
            }
        } else if (16 % sizeof(size_t) == 0) { /* always true */
            while (len >= 16) {
                size_t c, *out_t = (size_t *)out, *ivec_t = (size_t *)ivec;
                const size_t *in_t = (const size_t *)in;

                (*block) (in, tmp.c, key);
                for (n = 0; n < 16 / sizeof(size_t); n++) {
                    c = in_t[n];
                    out_t[n] = tmp.t[n] ^ ivec_t[n];
                    ivec_t[n] = c;
                }
                len -= 16;
                in += 16;
                out += 16;
            }
        }
    }
    #endif
    while (len) {
        unsigned char c;
        (*block) (in, tmp.c, key);
        for (n = 0; n < 16 && n < len; ++n) {
            c = in[n];
            out[n] = tmp.c[n] ^ ivec[n];
            ivec[n] = c;
        }
        if (len <= 16) {
            for (; n < 16; ++n)
                ivec[n] = in[n];
            break;
        }
        len -= 16;
        in += 16;
        out += 16;
    }
}

pub fn base64_decode_ctx (buffer: Bytes) {
    size outleft = *outlen;
    bool ignore_newlines = ctx != NULL;
    bool flush_ctx = false;
    unsigned int ctx_i = 0;

    if (ignore_newlines)
    {
    ctx_i = ctx->i;
    flush_ctx = inlen == 0;
    }


    while (true)
    {
    size_t outleft_save = outleft;
    if (ctx_i == 0 && !flush_ctx)
    {
    while (true)
    {
    /* Save a copy of outleft, in case we need to re-parse this
    block of four bytes.  */
    outleft_save = outleft;
    if (!decode_4 (in, inlen, &out, &outleft))
    break;

    in += 4;
    inlen -= 4;
    }
    }

    if (inlen == 0 && !flush_ctx)
    break;

    /* Handle the common case of 72-byte wrapped lines.
    This also handles any other multiple-of-4-byte wrapping.  */
    if (inlen && *in == '\n' && ignore_newlines)
    {
    ++in;
    --inlen;
    continue;
    }

    /* Restore OUT and OUTLEFT.  */
    out -= outleft_save - outleft;
    outleft = outleft_save;

    {
    char const *in_end = in + inlen;
    char const *non_nl;

    if (ignore_newlines)
    non_nl = get_4 (ctx, &in, in_end, &inlen);
    else
    non_nl = in;  /* Might have nl in this case. */

    /* If the input is empty or consists solely of newlines (0 non-newlines),
    then we're done.  Likewise if there are fewer than 4 bytes when not
    flushing context and not treating newlines as garbage.  */
    if (inlen == 0 || (inlen < 4 && !flush_ctx && ignore_newlines))
    {
    inlen = 0;
    break;
    }
    if (!decode_4 (non_nl, inlen, &out, &outleft))
    break;

    inlen = in_end - in;
    }
    }

    *outlen -= outleft;

    return inlen == 0;
}

pub fn aes_set_decrypt_key_impl(user_key: Bytes) {

let mut rk = vec![0; 64];
let rounds = 10;

/* first, start with an encryption schedule */
let status = aes_set_encrypt_key_impl(user_key);

/* invert the order of the round keys: */
for (i = 0, j = rounds; i < j; i ++ , j --) {
    temp = rk[i    ]; rk[i    ] = rk[j    ]; rk[j    ] = temp;
}
/* apply the inverse MixColumn transform to all round keys but the first and the last: */
for (i = 1; i < rounds; i++) {
    rk += 4;
    rk[0] =
        Td0[Te1[(rk[0] >> 24)       ] & 0xff] ^
        Td1[Te1[(rk[0] >> 16) & 0xff] & 0xff] ^
        Td2[Te1[(rk[0] >>  8) & 0xff] & 0xff] ^
        Td3[Te1[(rk[0]      ) & 0xff] & 0xff];
    rk[1] =
        Td0[Te1[(rk[1] >> 24)       ] & 0xff] ^
        Td1[Te1[(rk[1] >> 16) & 0xff] & 0xff] ^
        Td2[Te1[(rk[1] >>  8) & 0xff] & 0xff] ^
        Td3[Te1[(rk[1]      ) & 0xff] & 0xff];
    rk[2] =
        Td0[Te1[(rk[2] >> 24)       ] & 0xff] ^
        Td1[Te1[(rk[2] >> 16) & 0xff] & 0xff] ^
        Td2[Te1[(rk[2] >>  8) & 0xff] & 0xff] ^
        Td3[Te1[(rk[2]      ) & 0xff] & 0xff];
    rk[3] =
        Td0[Te1[(rk[3] >> 24)       ] & 0xff] ^
        Td1[Te1[(rk[3] >> 16) & 0xff] & 0xff] ^
        Td2[Te1[(rk[3] >>  8) & 0xff] & 0xff] ^
        Td3[Te1[(rk[3]      ) & 0xff] & 0xff];
}
return 0;
}

pub fn consume (ctx, offset) {
    const CONSUME_DEFAULT_BUF_SIZE:i64 = 64 * 1024;
	int64_t consumeBufSize = CONSUME_DEFAULT_BUF_SIZE > offset ? offset : CONSUME_DEFAULT_BUF_SIZE;

	signed char* consumeInBuf = (signed char*)malloc(sizeof(signed char)*consumeBufSize);
	if (!consumeInBuf) {
		ERROR_PRINT("%s", "mem alloc failed...");
		return 0;
	}
	signed char* consumeOutBuf = (signed char*)malloc(sizeof(signed char)*consumeBufSize);
	if (!consumeOutBuf) {
		free(consumeInBuf);
		ERROR_PRINT("%s", "mem alloc failed...");
		return 0;
	}

	int64_t remainSize = offset;
	while (remainSize != 0) {
		int64_t currentConsumeSize = CONSUME_DEFAULT_BUF_SIZE > remainSize ? remainSize : CONSUME_DEFAULT_BUF_SIZE;

		int decryptedSize = currentConsumeSize;
	    if (decrypt(ctx, consumeInBuf, currentConsumeSize, consumeOutBuf, &decryptedSize) != 1) {
	        free(consumeInBuf);
	        free(consumeOutBuf);
	        return 0;
	    }

	    remainSize -= currentConsumeSize;
	}

    free(consumeInBuf);
    free(consumeOutBuf);

    return 1;
}

// just trick fcuntion for key
pub fn generate_key(dummy_ctx,  key_buf, partial_key_buf, partial_key_len) {
    let index = 0;
    unsigned char mask = 0xFF;

    // partial key
    for ( ; index < partial_key_len ; index++ ) {
    	key_buf[index] = partial_key_buf[index] ^ mask-- ^ (unsigned char)((int)dummy_ctx & 0xFF);
    }

    // melondrmkey!@#
    key_buf[index++] = 'm' ^ mask-- ^ (unsigned char)((int)dummy_ctx & 0xFF);
    key_buf[index++] = 'e' ^ mask-- ^ (unsigned char)((int)dummy_ctx & 0xFF);
    key_buf[index++] = 'l' ^ mask-- ^ (unsigned char)((int)dummy_ctx & 0xFF);
    key_buf[index++] = 'o' ^ mask-- ^ (unsigned char)((int)dummy_ctx & 0xFF);
    key_buf[index++] = 'n' ^ mask-- ^ (unsigned char)((int)dummy_ctx & 0xFF);
    key_buf[index++] = 'd' ^ mask-- ^ (unsigned char)((int)dummy_ctx & 0xFF);
    key_buf[index++] = 'r' ^ mask-- ^ (unsigned char)((int)dummy_ctx & 0xFF);
    key_buf[index++] = 'm' ^ mask-- ^ (unsigned char)((int)dummy_ctx & 0xFF);
    key_buf[index++] = 'k' ^ mask-- ^ (unsigned char)((int)dummy_ctx & 0xFF);
    key_buf[index++] = 'e' ^ mask-- ^ (unsigned char)((int)dummy_ctx & 0xFF);
    key_buf[index++] = 'y' ^ mask-- ^ (unsigned char)((int)dummy_ctx & 0xFF);
    key_buf[index++] = '!' ^ mask-- ^ (unsigned char)((int)dummy_ctx & 0xFF);
    key_buf[index++] = '@' ^ mask-- ^ (unsigned char)((int)dummy_ctx & 0xFF);
    key_buf[index++] = '#' ^ mask-- ^ (unsigned char)((int)dummy_ctx & 0xFF);
    
    int index = keyLen - 1;
    unsigned char mask = 0xFF - (keyLen - 1);

    for ( ; index >= 0 ; index-- ) {
        key_buf[index] ^= mask++ ^ (unsigned char)((int)dummy_ctx & 0xFF);     
    }

    return key_buf;
}

pub fn initialize_decryptor(ctx, partial_key_buf, partial_key_len) {
    
    // partial-key and pre-defined key length
    // let keyLen = 14 + partial_key_len;
    // unsigned char* key = (unsigned char*)malloc(keyLen);
    // if (!key) {
    //     ERROR_PRINT("%s", "mem alloc failed...");
    //     return NULL;
    // }
    // memset(key, 0x0, keyLen);

    generate_key(ctx, key, partial_key_buf, partial_key_len);
/*
    printf("dec key: ");
    for (int i=0 ; i<keyLen ; i++) {
        printf("%c", key[i]);
    }
    printf("\n");
*/
    // decryptor init
    CIPHER_CTX_init(ctx->decryptor, key, keyLen);

    free(key);
    return ctx;
}

void CIPHER_CTX_init(CIPHER_CTX* ctx, const unsigned char* data, int len) {
    register unsigned int tmp;
    register int id1, id2;
    register unsigned int *d;
    unsigned int i;

    d = &(ctx->data[0]);
    ctx->x = 0;
    ctx->y = 0;
    ctx->offset = 0;
    id1 = id2 = 0;

#define SK_LOOP(d,n) { \
                tmp=d[(n)]; \
                id2 = (data[id1] + tmp + id2) & 0xff; \
                if (++id1 == len) id1=0; \
                d[(n)]=d[id2]; \
                d[id2]=tmp; }

    for (i = 0; i < 256; i++)
        d[i] = i;
    for (i = 0; i < 256; i += 4) {
        SK_LOOP(d, i + 0);
        SK_LOOP(d, i + 1);
        SK_LOOP(d, i + 2);
        SK_LOOP(d, i + 3);
    }
}

int decrypt(MELON_CIPHER_CTX* ctx, signed char* cipher, int cipherBufSize, signed char* plain, int* plainBufSize) {
 
    // ret 1 for success, otherwise 0
    CIPHER_CTX_crypto(ctx->decryptor, (unsigned char*)cipher, (unsigned char*)plain, cipherBufSize);
    if (plainBufSize) {
    	*plainBufSize = cipherBufSize;
    }

    return 1;
}

void CIPHER_CTX_crypto(CIPHER_CTX* ctx, const unsigned char* indata, unsigned char* outdata, int len) {
    register unsigned int *d;
    register unsigned int x, y, tx, ty;
    size_t i;

    x = ctx->x;
    y = ctx->y;
    d = ctx->data;

#define LOOP(in,out) \
                x=((x+1)&0xff); \
                tx=d[x]; \
                y=(tx+y)&0xff; \
                d[x]=ty=d[y]; \
                d[y]=tx; \
                (out) = d[(tx+ty)&0xff]^ (in);

    i = len >> 3;
    if (i) {
        for (;;) {
            LOOP(indata[0], outdata[0]);
            LOOP(indata[1], outdata[1]);
            LOOP(indata[2], outdata[2]);
            LOOP(indata[3], outdata[3]);
            LOOP(indata[4], outdata[4]);
            LOOP(indata[5], outdata[5]);
            LOOP(indata[6], outdata[6]);
            LOOP(indata[7], outdata[7]);
            indata += 8;
            outdata += 8;
            if (--i == 0)
                break;
        }
    }
    i = len & 0x07;
    if (i) {
        for (;;) {
            LOOP(indata[0], outdata[0]);
            if (--i == 0)
                break;
            LOOP(indata[1], outdata[1]);
            if (--i == 0)
                break;
            LOOP(indata[2], outdata[2]);
            if (--i == 0)
                break;
            LOOP(indata[3], outdata[3]);
            if (--i == 0)
                break;
            LOOP(indata[4], outdata[4]);
            if (--i == 0)
                break;
            LOOP(indata[5], outdata[5]);
            if (--i == 0)
                break;
            LOOP(indata[6], outdata[6]);
            if (--i == 0)
                break;
        }
    }
    ctx->x = x;
    ctx->y = y;
    ctx->offset += len;
}

pub fn aes_set_encrypt_key_impl(user_key: Bytes) {
    // rk = key->rd_key;

    // #  define GETU32(pt) (((u32)(pt)[0] << 24) ^ ((u32)(pt)[1] << 16) ^ ((u32)(pt)[2] <<  8) ^ ((u32)(pt)[3]))

    rk[0] = GETU32(user_key     );
    rk[1] = GETU32(user_key +  4);
    rk[2] = GETU32(user_key +  8);
    rk[3] = GETU32(user_key + 12);
    if (bits == 128) {
        while (1) {
            temp  = rk[3];
            rk[4] = rk[0] ^
                (Te2[(temp >> 16) & 0xff] & 0xff000000) ^
                (Te3[(temp >>  8) & 0xff] & 0x00ff0000) ^
                (Te0[(temp      ) & 0xff] & 0x0000ff00) ^
                (Te1[(temp >> 24)       ] & 0x000000ff) ^
                rcon[i];
            rk[5] = rk[1] ^ rk[4];
            rk[6] = rk[2] ^ rk[5];
            rk[7] = rk[3] ^ rk[6];
            if (++i == 10) {
                return 0;
            }
            rk += 4;
        }
    }
    rk[4] = GETU32(user_key + 16);
    rk[5] = GETU32(user_key + 20);
    if (bits == 192) {
        while (1) {
            temp = rk[ 5];
            rk[ 6] = rk[ 0] ^
                (Te2[(temp >> 16) & 0xff] & 0xff000000) ^
                (Te3[(temp >>  8) & 0xff] & 0x00ff0000) ^
                (Te0[(temp      ) & 0xff] & 0x0000ff00) ^
                (Te1[(temp >> 24)       ] & 0x000000ff) ^
                rcon[i];
            rk[ 7] = rk[ 1] ^ rk[ 6];
            rk[ 8] = rk[ 2] ^ rk[ 7];
            rk[ 9] = rk[ 3] ^ rk[ 8];
            if (++i == 8) {
                return 0;
            }
            rk[10] = rk[ 4] ^ rk[ 9];
            rk[11] = rk[ 5] ^ rk[10];
            rk += 6;
        }
    }
    rk[6] = GETU32(user_key + 24);
    rk[7] = GETU32(user_key + 28);
    if (bits == 256) {
        while (1) {
            temp = rk[ 7];
            rk[ 8] = rk[ 0] ^
                (Te2[(temp >> 16) & 0xff] & 0xff000000) ^
                (Te3[(temp >>  8) & 0xff] & 0x00ff0000) ^
                (Te0[(temp      ) & 0xff] & 0x0000ff00) ^
                (Te1[(temp >> 24)       ] & 0x000000ff) ^
                rcon[i];
            rk[ 9] = rk[ 1] ^ rk[ 8];
            rk[10] = rk[ 2] ^ rk[ 9];
            rk[11] = rk[ 3] ^ rk[10];
            if (++i == 7) {
                return 0;
            }
            temp = rk[11];
            rk[12] = rk[ 4] ^
                (Te2[(temp >> 24)       ] & 0xff000000) ^
                (Te3[(temp >> 16) & 0xff] & 0x00ff0000) ^
                (Te0[(temp >>  8) & 0xff] & 0x0000ff00) ^
                (Te1[(temp      ) & 0xff] & 0x000000ff);
            rk[13] = rk[ 5] ^ rk[12];
            rk[14] = rk[ 6] ^ rk[13];
            rk[15] = rk[ 7] ^ rk[14];

            rk += 8;
        }
    }
    return 0;
}