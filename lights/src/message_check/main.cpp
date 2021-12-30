#include <iostream>
#include <sstream>
#include <iomanip>
#include "messages.h"


std::string hexdump(const std::uint8_t* d, std::size_t length)
{
  std::stringstream ss;
  for (std::size_t i = 0; i < length; i++)
  {
    // std::setfill('0') << std::setw(2) << std::hex <<
    ss << "" <<  int{ d[i] } << ((i + 1 == length) ? "" : ", ");
  }
  return ss.str();
}

std::string hexdump(const Message& m)
{
  return hexdump(reinterpret_cast<const std::uint8_t*>(&m), sizeof(m));
}


Message empty()
{
  Message msg;
  msg.type = MsgType::NOP;
  msg._[0] = 0;
  msg._[1] = 0;
  msg._[2] = 0;
  for (std::size_t i = 0; i < sizeof(msg.raw); i++)
  {
    msg.raw[i] = 0;
  }
  return msg;
}

void print_config()
{
  Message msg = empty();
  msg.type = MsgType::CONFIG;
  msg.config.decay_time_delay_ms = 0xdeadbeef;
  msg.config.decay_interval_us = 0x01020304;
  msg.config.decay_amount = 0xF1F2F3F4;
  msg.config.gamma_r = 0.33333;
  msg.config.gamma_g = 1.0;
  msg.config.gamma_b = 0.0;
  std::cout << hexdump(msg) << std::endl;
}

int main(int argc, char* argv[])
{
  print_config();
}
