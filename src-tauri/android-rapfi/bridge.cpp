#include "source/command/command.h"
#include "source/core/iohelper.h"
#include <condition_variable>
#include <deque>
#include <iostream>
#include <mutex>
#include <streambuf>
#include <string>
#include <thread>

namespace {
std::mutex inputMutex;
std::condition_variable inputReady;
std::deque<char> inputQueue;
using OutputCallback = void (*)(const char *);
OutputCallback callback = nullptr;
std::mutex outputMutex;

class InputBuffer : public std::streambuf {
    char ch;
protected:
    int_type underflow() override {
        std::unique_lock<std::mutex> lock(inputMutex);
        inputReady.wait(lock, [] { return !inputQueue.empty(); });
        ch = inputQueue.front();
        inputQueue.pop_front();
        setg(&ch, &ch, &ch + 1);
        return traits_type::to_int_type(ch);
    }
};

class OutputBuffer : public std::streambuf {
    std::string line;
protected:
    int_type overflow(int_type c) override {
        if (traits_type::eq_int_type(c, traits_type::eof())) return traits_type::not_eof(c);
        std::lock_guard<std::mutex> lock(outputMutex);
        if (c == '\n') {
            if (callback) callback(line.c_str());
            line.clear();
        } else if (c != '\r') line.push_back(static_cast<char>(c));
        return c;
    }
    std::streamsize xsputn(const char *s, std::streamsize n) override {
        for (std::streamsize i = 0; i < n; ++i) overflow(s[i]);
        return n;
    }
};
InputBuffer input;
OutputBuffer output;
}

extern "C" int rapfi_bridge_start(const char *configPath, OutputCallback cb) {
    static bool started = false;
    if (started) return 0;
    if (!configPath || !cb) return -1;
    callback = cb;
    std::cin.rdbuf(&input);
    std::cout.rdbuf(&output);
    Command::configPath = configPath;
    Command::allowInternalConfig = false;
    if (!Command::loadConfig()) return -2;
    started = true;
    std::thread([] { Command::gomocupLoop(); }).detach();
    return 0;
}

extern "C" void rapfi_bridge_send(const char *command) {
    if (!command) return;
    {
        std::lock_guard<std::mutex> lock(inputMutex);
        for (const char *p = command; *p; ++p) inputQueue.push_back(*p);
        inputQueue.push_back('\n');
    }
    inputReady.notify_one();
}
