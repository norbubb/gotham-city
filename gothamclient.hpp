#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

static const uintptr_t SEGMENT_SIZE = 8;

static const uintptr_t NUM_SEGMENTS = 32;

extern "C" {

void recv_android_path(const char *android_path);

int32_t add_method(int32_t nx, int32_t ny);

char *contact_with_str(const char *inx);

long long create_client(const char *endpoint);

long long create_wallet(long long cclient_shim_num_ptr, const char *wallet_name);

char *drive_new_address_wallet(const char *wallet_name);

const char *wallet_path(const char *wallet_name);

long long load_wallet(const char *wallet_name);

char *simple_sign_message(const char *msg,
                          const char *wallet_name,
                          const char *address,
                          long long cclient_shim_num_ptr);

const char *eth_enter(long long cclient_shim_num_ptr, const char *wallet_name, const char *address);

/// # Safety
///
/// - This function should only be called with valid C pointers.
/// - Arguments are accessed in arbitrary locations.
/// - Strings should be null terminated array of bytes.
char *get_client_master_key(const char *c_endpoint, const char *c_auth_token);

jstring Java_com_zengo_components_kms_gotham_ECDSA_getClientMasterKey(JNIEnv env,
                                                                      JClass _class,
                                                                      JString j_endpoint,
                                                                      JString j_auth_token);

char *decrypt_party_one_master_key(const char *c_master_key_two_json,
                                   const char *c_helgamal_segmented_json,
                                   const char *c_private_key);

char *get_child_mk1(const char *c_master_key_one_json, int32_t c_x_pos, int32_t c_y_pos);

char *get_child_mk2(const char *c_master_key_two_json, int32_t c_x_pos, int32_t c_y_pos);

char *construct_single_private_key(const char *c_mk1_x1, const char *c_mk2_x2);

jstring Java_com_zengo_components_kms_gotham_ECDSA_decryptPartyOneMasterKey(JNIEnv env,
                                                                            JClass _class,
                                                                            JString j_master_key_two_json,
                                                                            JString j_helgamal_segmented_json,
                                                                            JString j_private_key);

jstring Java_com_zengo_components_kms_gotham_ECDSA_getChildMk2(JNIEnv env,
                                                               JClass _class,
                                                               JString j_master_key_two_json,
                                                               jint j_x_pos,
                                                               jint j_y_pos);

jstring Java_com_zengo_components_kms_gotham_ECDSA_getChildMk1(JNIEnv env,
                                                               JClass _class,
                                                               JString j_master_key_one_json,
                                                               jint j_x_pos,
                                                               jint j_y_pos);

jstring Java_com_zengo_components_kms_gotham_ECDSA_constructSinglePrivateKey(JNIEnv env,
                                                                             JClass _class,
                                                                             JString j_mk1_x1,
                                                                             JString j_mk2_x2);

/// # Safety
///
/// - This function should only be called with valid C pointers.
/// - Arguments are accessed in arbitrary locations.
/// - Strings should be null terminated array of bytes.
char *sign_message(const char *c_endpoint,
                   const char *c_auth_token,
                   const char *c_message_le_hex,
                   const char *c_master_key_json,
                   int32_t c_x_pos,
                   int32_t c_y_pos,
                   const char *c_id);

jstring Java_com_zengo_components_kms_gotham_ECDSA_signMessage(JNIEnv env,
                                                               JClass _class,
                                                               JString j_endpoint,
                                                               JString j_auth_token,
                                                               JString j_message_le_hex,
                                                               JString j_master_key_json,
                                                               jint j_x_pos,
                                                               jint j_y_pos,
                                                               JString j_id);

} // extern "C"
