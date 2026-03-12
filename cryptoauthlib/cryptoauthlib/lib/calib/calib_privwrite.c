/**
 * \file
 * \brief CryptoAuthLib Basic API methods for PrivWrite command.
 *
 * The PrivWrite command is used to write externally generated ECC private keys
 * into the device.
 *
 * \note List of devices that support this command - ATECC108A, ATECC508A, and
 *       ATECC608A/B. There are differences in the modes that they support. Refer
 *       to device datasheets for full details.
 *
 * \copyright (c) 2015-2020 Microchip Technology Inc. and its subsidiaries.
 *
 * \page License
 *
 * Subject to your compliance with these terms, you may use Microchip software
 * and any derivatives exclusively with Microchip products. It is your
 * responsibility to comply with third party license terms applicable to your
 * use of third party software (including open source software) that may
 * accompany Microchip software.
 *
 * THIS SOFTWARE IS SUPPLIED BY MICROCHIP "AS IS". NO WARRANTIES, WHETHER
 * EXPRESS, IMPLIED OR STATUTORY, APPLY TO THIS SOFTWARE, INCLUDING ANY IMPLIED
 * WARRANTIES OF NON-INFRINGEMENT, MERCHANTABILITY, AND FITNESS FOR A
 * PARTICULAR PURPOSE. IN NO EVENT WILL MICROCHIP BE LIABLE FOR ANY INDIRECT,
 * SPECIAL, PUNITIVE, INCIDENTAL OR CONSEQUENTIAL LOSS, DAMAGE, COST OR EXPENSE
 * OF ANY KIND WHATSOEVER RELATED TO THE SOFTWARE, HOWEVER CAUSED, EVEN IF
 * MICROCHIP HAS BEEN ADVISED OF THE POSSIBILITY OR THE DAMAGES ARE
 * FORESEEABLE. TO THE FULLEST EXTENT ALLOWED BY LAW, MICROCHIP'S TOTAL
 * LIABILITY ON ALL CLAIMS IN ANY WAY RELATED TO THIS SOFTWARE WILL NOT EXCEED
 * THE AMOUNT OF FEES, IF ANY, THAT YOU HAVE PAID DIRECTLY TO MICROCHIP FOR
 * THIS SOFTWARE.
 */

#include "cryptoauthlib.h"

#if CALIB_PRIVWRITE_EN

#include "host/atca_host.h"

#if (CA_MAX_PACKET_SIZE < PRIVWRITE_COUNT)
#error "PrivWrite command packet cannot be accommodated inside the maximum packet size provided"
#endif

/** \brief Executes PrivWrite command, to write externally generated ECC
 *          private keys into the device.
 *
 *  \param[in] device        Device context pointer
 *  \param[in] key_id        Slot to write the external private key into.
 *  \param[in] priv_key      External private key (36 bytes) to be written.
 *                           The first 4 bytes should be zero for P256 curve.
 *  \param[in] write_key_id  Write key slot. Ignored if write_key is NULL.
 *  \param[in] write_key     Write key (32 bytes). If NULL, perform an
 *                           unencrypted PrivWrite, which is only available when
 *                           the data zone is unlocked.
 *  \param[in]  num_in       20 byte host nonce to inject into Nonce calculation
 *
 *  \return ATCA_SUCCESS on success, otherwise an error code.
 */
#if defined(ATCA_USE_CONSTANT_HOST_NONCE)
ATCA_STATUS calib_priv_write(ATCADevice device, uint16_t key_id, const uint8_t priv_key[36], uint16_t write_key_id, const uint8_t write_key[32])
{
    uint8_t num_in[NONCE_NUMIN_SIZE] = { 0 };

#else
ATCA_STATUS calib_priv_write(ATCADevice device, uint16_t key_id, const uint8_t priv_key[36], uint16_t write_key_id, const uint8_t write_key[32],
                             const uint8_t num_in[NONCE_NUMIN_SIZE])
{
#endif
    ATCAPacket packet;
    ATCA_STATUS status;
    atca_nonce_in_out_t nonce_params;
    atca_gen_dig_in_out_t gen_dig_param;
    atca_write_mac_in_out_t host_mac_param;
    atca_temp_key_t temp_key;
    uint8_t serial_num[32]; // Buffer is larger than the 9 bytes required to make reads easier
    uint8_t rand_out[RANDOM_NUM_SIZE] = { 0 };
    uint8_t cipher_text[36] = { 0 };
    uint8_t host_mac[MAC_SIZE] = { 0 };
    uint8_t other_data[4] = { 0 };

    if ((device == NULL) || (priv_key == NULL) || (key_id > 15u))
    {
        return ATCA_TRACE(ATCA_BAD_PARAM, "Either NULL pointer or invalid slot received");
    }

    do
    {
        (void)memset(&packet, 0x00, sizeof(ATCAPacket));

        if (write_key == NULL)
        {
            // Caller requested an unencrypted PrivWrite, which is only allowed when the data zone is unlocked
            // build an PrivWrite command
            packet.param1 = 0x00;                           // Mode is unencrypted write
            packet.param2 = key_id;                         // Key ID
            (void)memcpy(&packet.data[0], priv_key, 36);    // Private key
            (void)memset(&packet.data[36], 0, 32);          // MAC (ignored for unencrypted write)
        }
        else
        {
            // Read the device SN
            if ((status = calib_read_zone(device, ATCA_ZONE_CONFIG, 0, 0, 0, serial_num, 32)) != ATCA_SUCCESS)
            {
                (void)ATCA_TRACE(status, "calib_read_zone - failed");
                break;
            }
            // Make the SN continuous by moving SN[4:8] right after SN[0:3]
            (void)memmove(&serial_num[4], &serial_num[8], 5);

            // Send the random Nonce command
            if ((status = calib_nonce_rand(device, num_in, rand_out)) != ATCA_SUCCESS)
            {
                (void)ATCA_TRACE(status, "calib_nonce_rand - failed");
                break;
            }

            // Calculate Tempkey
            (void)memset(&temp_key, 0, sizeof(temp_key));
            (void)memset(&nonce_params, 0, sizeof(nonce_params));
            nonce_params.mode = NONCE_MODE_SEED_UPDATE;
            nonce_params.zero = 0;
            nonce_params.num_in = &num_in[0];
            nonce_params.rand_out = rand_out;
            nonce_params.temp_key = &temp_key;
            if ((status = atcah_nonce(&nonce_params)) != ATCA_SUCCESS)
            {
                (void)ATCA_TRACE(status, "atcah_nonce - failed");
                break;
            }

            // Supply OtherData so GenDig behavior is the same for keys with SlotConfig.NoMac set
            other_data[0] = ATCA_GENDIG;
            other_data[1] = GENDIG_ZONE_DATA;
            other_data[2] = (uint8_t)(write_key_id & 0xFFu);
            other_data[3] = (uint8_t)(write_key_id >> 8u);

            // Send the GenDig command
            if ((status = calib_gendig(device, GENDIG_ZONE_DATA, write_key_id, other_data, (uint8_t)sizeof(other_data))) != ATCA_SUCCESS)
            {
                (void)ATCA_TRACE(status, "calib_gendig - failed");
                break;
            }

            // Calculate Tempkey
            // NoMac bit isn't being considered here on purpose to remove having to read SlotConfig.
            // OtherData is built to get the same result regardless of the NoMac bit.
            (void)memset(&gen_dig_param, 0, sizeof(gen_dig_param));
            gen_dig_param.zone = GENDIG_ZONE_DATA;
            gen_dig_param.sn = serial_num;
            gen_dig_param.key_id = write_key_id;
            gen_dig_param.is_key_nomac = false;
            gen_dig_param.stored_value = write_key;
            gen_dig_param.other_data = other_data;
            gen_dig_param.temp_key = &temp_key;
            if ((status = atcah_gen_dig(&gen_dig_param)) != ATCA_SUCCESS)
            {
                (void)ATCA_TRACE(status, "atcah_gen_dig - failed");
                break;
            }

            // Calculate Auth MAC and cipher text
            (void)memset(&host_mac_param, 0, sizeof(host_mac_param));
            host_mac_param.zone = PRIVWRITE_MODE_ENCRYPT;
            host_mac_param.key_id = key_id;
            host_mac_param.sn = serial_num;
            host_mac_param.input_data = &priv_key[0];
            host_mac_param.encrypted_data = cipher_text;
            host_mac_param.auth_mac = host_mac;
            host_mac_param.temp_key = &temp_key;
            if ((status = atcah_privwrite_auth_mac(&host_mac_param)) != ATCA_SUCCESS)
            {
                (void)ATCA_TRACE(status, "atcah_privwrite_auth_mac - failed");
                break;
            }

            // build a write command for encrypted writes
            packet.param1 = PRIVWRITE_MODE_ENCRYPT; // Mode is encrypted write
            packet.param2 = key_id;                 // Key ID
            (void)memcpy(&packet.data[0], cipher_text, sizeof(cipher_text));
            (void)memcpy(&packet.data[sizeof(cipher_text)], host_mac, sizeof(host_mac));
        }

        if ((status = atPrivWrite(atcab_get_device_type_ext(device), &packet)) != ATCA_SUCCESS)
        {
            (void)ATCA_TRACE(status, "atPrivWrite - failed");
            break;
        }

        if ((status = atca_execute_command(&packet, device)) != ATCA_SUCCESS)
        {
            (void)ATCA_TRACE(status, "calib_priv_write - execution failed");
            break;
        }

    } while (false);

    return status;
}
#endif  /* CALIB_PRIVWRITE_EN */

ATCA_STATUS atcah_nonce(struct atca_nonce_in_out *param)
{
    uint8_t temporary[ATCA_MSG_SIZE_NONCE], nonce_numin_size;
    uint8_t *p_temp;
    uint8_t calc_mode = param->mode & NONCE_MODE_MASK;
    ATCADeviceType device_type = atcab_get_device_type();

    // Check parameters
    if (param->temp_key == NULL || param->num_in == NULL)
    {
        return ATCA_BAD_PARAM;
    }

    (void)calib_get_numin_size(calc_mode, &nonce_numin_size);

    // Calculate or pass-through the nonce to TempKey->Value
    if ((calc_mode == NONCE_MODE_SEED_UPDATE) || (calc_mode == NONCE_MODE_NO_SEED_UPDATE))
    {
        // RandOut is only required for these modes
        if (param->rand_out == NULL)
        {
            return ATCA_BAD_PARAM;
        }

        if ((param->zero & NONCE_ZERO_CALC_MASK) == NONCE_ZERO_CALC_TEMPKEY)
        {
            // Nonce calculation mode. Actual value of TempKey has been returned in RandOut
            (void)memcpy(param->temp_key->value, param->rand_out, 32);

            // TempKey flags aren't changed
        }
        else
        {
            // Calculate nonce using SHA-256 (refer to data sheet)
            p_temp = temporary;

            (void)memcpy(p_temp, param->rand_out, RANDOM_NUM_SIZE);
            p_temp += RANDOM_NUM_SIZE;

            (void)memcpy(p_temp, param->num_in, nonce_numin_size);
            p_temp += nonce_numin_size;

            *p_temp++ = ATCA_NONCE;
            *p_temp++ = param->mode;
            *p_temp++ = 0x00;

            // Calculate SHA256 to get the nonce
            (void)atcac_sw_sha2_256(temporary, ATCA_MSG_SIZE_NONCE, param->temp_key->value);

            // Update TempKey flags
            if ((SHA104 == device_type) || (SHA105 == device_type))
            {
                param->temp_key->source_flag = 0; // Random
            }
            else
            {
                param->temp_key->source_flag = 0; // Random
                param->temp_key->key_id = 0;
                param->temp_key->gen_dig_data = 0;
                param->temp_key->no_mac_flag = 0;
                param->temp_key->valid = 1;
            }
        }

        // Update TempKey to only 32 bytes
        param->temp_key->is_64 = 0;
    }
    else if ((param->mode & NONCE_MODE_MASK) == NONCE_MODE_PASSTHROUGH)
    {

        if ((param->mode & NONCE_MODE_TARGET_MASK) == NONCE_MODE_TARGET_TEMPKEY)
        {
            (void)memcpy(param->temp_key->value, param->num_in, nonce_numin_size);

            // Pass-through mode for TempKey (other targets have no effect on TempKey)
            if ((param->mode & NONCE_MODE_INPUT_LEN_MASK) == NONCE_MODE_INPUT_LEN_64)
            {
                param->temp_key->is_64 = 1;
            }
            else
            {
                param->temp_key->is_64 = 0;
            }

            // Update TempKey flags
            if ((SHA104 == device_type) || (SHA105 == device_type))
            {
                param->temp_key->source_flag = 1; // Not Random
            }
            else
            {
                param->temp_key->source_flag = 1; // Not Random
                param->temp_key->key_id = 0;
                param->temp_key->gen_dig_data = 0;
                param->temp_key->no_mac_flag = 0;
                param->temp_key->valid = 1;
            }
        }
        else //In the case of ECC608, passthrough message may be stored in message digest buffer/ Alternate Key buffer
        {

            // Update TempKey flags
            param->temp_key->source_flag = 1; //Not Random
            param->temp_key->key_id = 0;
            param->temp_key->gen_dig_data = 0;
            param->temp_key->no_mac_flag = 0;
            param->temp_key->valid = 0;

        }
    }
    else if ((NONCE_MODE_GEN_SESSION_KEY == calc_mode) && (param->zero >= 0x8000u))
    {
        // Calculate nonce using SHA-256 (refer to data sheet)
        p_temp = temporary;

        (void)memcpy(p_temp, param->rand_out, RANDOM_NUM_SIZE);
        p_temp += RANDOM_NUM_SIZE;

        (void)memcpy(p_temp, param->num_in, nonce_numin_size);
        p_temp += nonce_numin_size;

        *p_temp++ = ATCA_NONCE;
        *p_temp++ = param->mode;
        *p_temp++ = (uint8_t)((param->zero) & 0xFFu);

        // Calculate SHA256 to get the nonce
        (void)atcac_sw_sha2_256(temporary, ATCA_MSG_SIZE_NONCE, param->temp_key->value);

        if ((SHA104 == device_type) || (SHA105 == device_type))
        {
            param->temp_key->source_flag = 0;
        }
    }
    else
    {
        return ATCA_BAD_PARAM;
    }

    return ATCA_SUCCESS;
}

ATCA_STATUS atcah_privwrite_auth_mac(struct atca_write_mac_in_out *param)
{
    uint8_t mac_input[ATCA_MSG_SIZE_PRIVWRITE_MAC];
    uint8_t i = 0;
    uint8_t *p_temp = NULL;
    uint8_t session_key2[32] = { 0 };

    // Check parameters
    if ((NULL == param->input_data) || (NULL == param->temp_key))
    {
        return ATCA_BAD_PARAM;
    }

    // Check TempKey fields validity (TempKey is always used)
    if ( // TempKey.CheckFlag must be 0 and TempKey.Valid must be 1
        (0u != param->temp_key->no_mac_flag) || (param->temp_key->valid != 1u)
        )
    {
        // Invalidate TempKey, then return
        param->temp_key->valid = 0;
        return ATCA_EXECUTION_ERROR;
    }


    /* Encrypt by XOR-ing Data with the TempKey
     */

    // Encrypt the next 28 bytes of the cipher text, which is the first part of the private key.
    for (i = 0u; i < 32u; i++)
    {
        param->encrypted_data[i] = param->input_data[i] ^ param->temp_key->value[i];
    }

    // Calculate the new key for the last 4 bytes of the cipher text
    (void)atcac_sw_sha2_256(param->temp_key->value, 32, session_key2);

    // Encrypt the last 4 bytes of the cipher text, which is the remaining part of the private key
    for (i = 32u; i < 36u; i++)
    {
        param->encrypted_data[i] = param->input_data[i] ^ session_key2[i - 32u];
    }

    // If the pointer *mac is provided by the caller then calculate input MAC
    if (NULL != param->auth_mac)
    {
        // Start calculation
        p_temp = mac_input;

        // (1) 32 bytes TempKey
        (void)memcpy(p_temp, param->temp_key->value, ATCA_KEY_SIZE);
        p_temp += ATCA_KEY_SIZE;

        // (2) 1 byte Opcode
        *p_temp++ = ATCA_PRIVWRITE;

        // (3) 1 byte Param1 (zone)
        *p_temp++ = param->zone;

        // (4) 2 bytes Param2 (keyID)
        *p_temp++ = (uint8_t)(param->key_id & 0xFFu);
        *p_temp++ = (uint8_t)((param->key_id >> 8u) & 0xFFu);

        // (5) 1 byte SN[8]
        *p_temp++ = param->sn[8];

        // (6) 2 bytes SN[0:1]
        *p_temp++ = param->sn[0];
        *p_temp++ = param->sn[1];

        // (7) 21 zeros
        (void)memset(p_temp, 0, ATCA_PRIVWRITE_MAC_ZEROS_SIZE);
        p_temp += ATCA_PRIVWRITE_MAC_ZEROS_SIZE;

        // (8) 36 bytes PlainText (Private Key)
        (void)memcpy(p_temp, param->input_data, ATCA_PRIVWRITE_PLAIN_TEXT_SIZE);

        // Calculate SHA256 to get the new TempKey
        (void)atcac_sw_sha2_256(mac_input, sizeof(mac_input), param->auth_mac);
    }

    return ATCA_SUCCESS;
}

ATCA_STATUS atcah_gen_dig(struct atca_gen_dig_in_out *param)
{
    uint8_t temporary[ATCA_MSG_SIZE_GEN_DIG];
    uint8_t *p_temp;

    // Check parameters
    if (param->sn == NULL || param->temp_key == NULL)
    {
        return ATCA_BAD_PARAM;
    }
    if ((param->zone <= GENDIG_ZONE_DATA) && (param->stored_value == NULL))
    {
        return ATCA_BAD_PARAM;  // Stored value cannot be null for Config,OTP and Data
    }

    if ((param->zone == GENDIG_ZONE_SHARED_NONCE || (param->zone == GENDIG_ZONE_DATA && param->is_key_nomac)) && param->other_data == NULL)
    {
        return ATCA_BAD_PARAM;  // Other data is required in these cases
    }

    if (param->zone > 5u)
    {
        return ATCA_BAD_PARAM;  // Unknown zone

    }
    // Start calculation
    p_temp = temporary;


    // (1) 32 bytes inputKey
    if (param->zone == GENDIG_ZONE_SHARED_NONCE)
    {
        if (GENDIG_USE_TEMPKEY_BIT == (param->key_id & GENDIG_USE_TEMPKEY_BIT))
        {
            (void)memcpy(p_temp, param->temp_key->value, ATCA_KEY_SIZE);  // 32 bytes TempKey
        }
        else
        {
            (void)memcpy(p_temp, param->other_data, ATCA_KEY_SIZE);       // 32 bytes other data

        }
    }
    else if (param->zone == GENDIG_ZONE_COUNTER || param->zone == GENDIG_ZONE_KEY_CONFIG)
    {
        (void)memset(p_temp, 0x00, ATCA_KEY_SIZE);                        // 32 bytes of zero.

    }
    else
    {
        (void)memcpy(p_temp, param->stored_value, ATCA_KEY_SIZE);     // 32 bytes of stored data

    }

    p_temp += ATCA_KEY_SIZE;


    if (param->zone == GENDIG_ZONE_DATA && param->is_key_nomac)
    {
        // If a key has the SlotConfig.NoMac bit set, then opcode and parameters come from OtherData
        (void)memcpy(p_temp, param->other_data, 4);
        p_temp += 4;
    }
    else
    {
        // (2) 1 byte Opcode
        *p_temp++ = ATCA_GENDIG;

        // (3) 1 byte Param1 (zone)
        *p_temp++ = param->zone;

        // (4) 1 byte LSB of Param2 (keyID)
        *p_temp++ = (uint8_t)(param->key_id & 0xFFu);
        if (param->zone == GENDIG_ZONE_SHARED_NONCE)
        {
            //(4) 1 byte zero for shared nonce mode
            *p_temp++ = 0;
        }
        else
        {
            //(4)  1 byte MSB of Param2 (keyID) for other modes
            *p_temp++ = (uint8_t)(param->key_id >> 8);
        }
    }

    // (5) 1 byte SN[8]
    *p_temp++ = param->sn[8];

    // (6) 2 bytes SN[0:1]
    *p_temp++ = param->sn[0];
    *p_temp++ = param->sn[1];


    // (7)
    if (param->zone == GENDIG_ZONE_COUNTER)
    {
        *p_temp++ = 0;
        *p_temp++ = (uint8_t)(param->counter & 0xFFu);   // (7) 4 bytes of counter
        *p_temp++ = (uint8_t)((param->counter >> 8u) & 0xFFu);
        *p_temp++ = (uint8_t)((param->counter >> 16u) & 0xFFu);
        *p_temp++ = (uint8_t)((param->counter >> 24u) & 0xFFu);

        (void)memset(p_temp, 0x00, 20);                       // (7) 20 bytes of zero
        p_temp += 20;

    }
    else if (param->zone == GENDIG_ZONE_KEY_CONFIG)
    {
        *p_temp++ = 0;
        *p_temp++ = (uint8_t)(param->slot_conf & 0xFFu);            // (7) 2 bytes of Slot config
        *p_temp++ = (uint8_t)(param->slot_conf >> 8u);

        *p_temp++ = (uint8_t)(param->key_conf & 0xFFu);
        *p_temp++ = (uint8_t)(param->key_conf >> 8u);  // (7) 2 bytes of key config

        *p_temp++ = param->slot_locked;                // (7) 1 byte of slot locked

        (void)memset(p_temp, 0x00, 19);                // (7) 19 bytes of zero
        p_temp += 19;

    }
    else
    {

        (void)memset(p_temp, 0, ATCA_GENDIG_ZEROS_SIZE);       // (7) 25 zeros
        p_temp += ATCA_GENDIG_ZEROS_SIZE;

    }

    if (param->zone == GENDIG_ZONE_SHARED_NONCE && (0x8000u == (param->key_id & 0x8000u)))
    {
        (void)memcpy(p_temp, param->other_data, ATCA_KEY_SIZE);           // (8) 32 bytes OtherData
        p_temp += ATCA_KEY_SIZE;

    }
    else
    {
        (void)memcpy(p_temp, param->temp_key->value, ATCA_KEY_SIZE);      // (8) 32 bytes TempKey
        p_temp += ATCA_KEY_SIZE;

    }

    // Calculate SHA256 to get the new TempKey
    (void)atcac_sw_sha2_256(temporary, atcab_pointer_delta(p_temp, temporary), param->temp_key->value);

    // Update TempKey fields
    param->temp_key->valid = 1;

    if ((param->zone == GENDIG_ZONE_DATA) && (param->key_id <= 15u))
    {
        param->temp_key->gen_dig_data = 1;
        param->temp_key->key_id = (uint8_t)(param->key_id & 0xFu);   // mask lower 4-bit only
        if (param->is_key_nomac)
        {
            param->temp_key->no_mac_flag = 1;
        }
    }
    else
    {
        param->temp_key->gen_dig_data = 0;
        param->temp_key->key_id = 0;
    }

    return ATCA_SUCCESS;
}
