// Copyright (C) 2012 Risto Saarelma

#include "static_file.hpp"

extern "C" {
extern char *stbi_zlib_decode_malloc(char const *buffer, int len, int *outlen);
}

Static_File* Static_File::s_root = nullptr;

Static_File::Static_File(std::string name, int len, bool compressed, const unsigned char* data)
    : name(name)
    , len(len)
    , data(data)
    , own_data(false)
    , next(Static_File::s_root)
{
  Static_File::s_root = this;

  if (compressed)
    decompress();
}

Static_File::~Static_File() {
  if (own_data)
    delete data;
}

Static_File* Static_File::find(const std::string name) {
  for (Static_File* ptr = Static_File::s_root; ptr; ptr = ptr->next) {
    if (ptr->get_name() == name)
      return ptr;
  }
  return nullptr;
}

void Static_File::decompress() {
  char* new_data;
  int new_len;
  new_data = stbi_zlib_decode_malloc(
      reinterpret_cast<const char*>(data), len, &new_len);
  own_data = true;
  data = reinterpret_cast<const unsigned char*>(new_data);
  len = new_len;
}
