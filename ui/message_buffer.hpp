#ifndef MESSAGE_BUFFER_HPP
#define MESSAGE_BUFFER_HPP

#include <string>
#include <queue>
#include <list>

struct Message_String {
  std::string text;
  float time_read;
};

class Message_Buffer {
 public:
  Message_Buffer();
  void update(float interval_seconds);
  void draw();
  void add_msg(std::string str);
  void add_caption(std::string str);
 private:
  // Update the total time when texts will be read and return the time
  // the user should have read added_text.
  float time_read(std::string added_text);

  // Current time in seconds.
  float clock;
  // The estimated time when the user will have finished reading all the text
  // currently on screen. Either equal to clock or larger than it.
  float read_new_text_time;
  float letter_read_duration;
  std::list<Message_String> messages;
  std::queue<Message_String> captions;
};

#endif
