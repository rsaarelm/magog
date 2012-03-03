#ifndef UTIL_STATIC_FILE_HPP
#define UTIL_STATIC_FILE_HPP

#include <string>

class Static_File {
 public:
  Static_File(std::string name, int len, bool compressed, const unsigned char* data);
  ~Static_File();

  const unsigned char* get_data() const { return data; }
  int get_len() const { return len; }
  std::string get_name() const { return name; }

  static Static_File* find(const std::string name);
 private:
  Static_File(const Static_File&);
  Static_File& operator=(const Static_File&);

  void decompress();

  std::string name;
  int len;
  const unsigned char* data;
  bool own_data;

  static Static_File* s_root;
  Static_File* next;
};

// Produce junk global variables just for the purpose of pushing their values
// into the static root index in Static_File.
#define _UTIL_MERGE(x, y) x##y
#define _UTIL_LABEL(a) _UTIL_MERGE(_util_gen_file_,a)

#define UTIL_COMPRESSED_FILE(name, len, data) \
  static Static_File _UTIL_LABEL(__LINE__)(name, len, true, reinterpret_cast<const unsigned char*>(data));

#define UTIL_FILE(name, len, data) \
  static Static_File _UTIL_LABEL(__LINE__)(name, len, false, reinterpret_cast<const unsigned char*>(data));

#endif
