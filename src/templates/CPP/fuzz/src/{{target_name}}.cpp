#include <cstdint>
#include <cstddef>
#include <cstdlib>
#include <cstring>
#include <string>
#include <memory>
{{#if detected_xml_lib}}
#include <sstream>
{{/if}}

{{#unless minimal}}
#include "lib.h"
{{else}}
{{#if detected_xml_lib}}
// Auto-detected XML parsing library - add your specific headers
// TODO: Uncomment the headers for your specific XML library:
// #include "adm/parse.hpp"          // For libadm
// #include "adm/document.hpp"       // For libadm
// #include "rapidxml/rapidxml.hpp"  // For RapidXML
// #include "pugixml.hpp"            // For PugiXML
// #include "tinyxml2.h"             // For TinyXML2
#include <iostream>  // For demonstration
{{else}}
{{#if suggested_targets}}
// Project-specific headers based on analysis
#include <adm/parse.hpp>
{{else}}
// TODO: Replace this placeholder with your project's header files
// Based on your project structure, you might need:
// #include "your_lib/parser.h"
// #include "your_lib/processor.h"
#include <iostream>  // For demonstration crash
{{/if}}
{{/if}}
{{/unless}}

// C++ name mangling prevention - required when using C driver
// The 'extern "C"' linkage is essential for C++ fuzz harnesses to work with
// the universal C driver (driver/main.c) used by AFL, HonggFuzz, and standalone modes.
// LibFuzzer doesn't need this because it provides its own main() function.
extern "C" int LLVMFuzzerTestOneInput(const uint8_t* data, size_t size) {
{{#unless minimal}}
    // Example: Process the input data through your library function
    // Note: Ensure data is properly null-terminated if your function expects a string
    if (size > 0) {
        // Create a mutable copy for the C library function (which expects char*)
        std::unique_ptr<char[]> input_copy(new char[size + 1]);
        std::memcpy(input_copy.get(), data, size);
        input_copy[size] = '\0';  // Null-terminate
        
        try {
            process(input_copy.get());
        } catch (const std::exception& e) {
            // Exceptions are handled by libFuzzer - they're treated as interesting inputs
            // but not crashes (unless AddressSanitizer detects memory errors)
        }
    }
{{else}}
{{#if suggested_targets}}
    // ========================================================================
    // PROJECT-SPECIFIC FUZZING HARNESS
    // ========================================================================
    // Generated from analysis of {{project_name}}
    
    // Skip empty inputs
    if (!data || size == 0) return 0;
    
    // Convert to string format for most parsing functions
    std::string input(reinterpret_cast<const char*>(data), size);
    
    try {
        {{{suggested_example_code}}}
        
    } catch (const std::exception& e) {
        // Expected for malformed input - don't crash the fuzzer
        return 0;
    } catch (...) {
        // Catch any other exceptions
        return 0;
    }
{{else}}
{{#if detected_xml_lib}}
    // ========================================================================
    // XML PARSING HARNESS (GENERIC)
    // ========================================================================
    // Auto-detected XML parsing library - customize for your specific API
    
    // Skip empty inputs
    if (!data || size == 0) return 0;
    
    // Convert to string format expected by most XML parsers
    std::string xml_input(reinterpret_cast<const char*>(data), size);
    
    try {
        // TODO: Replace with your XML library's parsing function
        // Common patterns:
        // 
        // For RapidXML:
        // rapidxml::xml_document<> doc;
        // std::vector<char> buffer(xml_input.begin(), xml_input.end());
        // buffer.push_back('\0');
        // doc.parse<0>(&buffer[0]);
        //
        // For PugiXML:
        // pugi::xml_document doc;
        // pugi::xml_parse_result result = doc.load_string(xml_input.c_str());
        //
        // For TinyXML2:
        // tinyxml2::XMLDocument doc;
        // doc.Parse(xml_input.c_str());
        
        // Placeholder - replace with your actual parsing call
        std::cout << "Parsing XML of length: " << size << std::endl;
        
    } catch (const std::exception& e) {
        // Expected for malformed XML - just return
    } catch (...) {
        // Catch any other exceptions
    }
{{else}}
    // ========================================================================
    // GENERIC FUZZING HARNESS
    // ========================================================================
    // Customize this harness for your specific library and use case
    
    // Skip empty inputs
    if (!data || size == 0) return 0;
    
    try {
        // TODO: Replace with your library's main entry points
        // 
        // Common patterns:
        // 1. String processing:
        //    std::string input(reinterpret_cast<const char*>(data), size);
        //    your_string_processor(input);
        //
        // 2. Binary data processing:
        //    your_binary_processor(data, size);
        //
        // 3. Protocol parsing:
        //    your_protocol_parser(data, size);
        //
        // 4. File format parsing:
        //    MemoryStream stream(data, size);
        //    your_file_parser(stream);
        
        // Simple demonstration: crash if input contains "bug"
        // TODO: Remove this demonstration code once you add real logic
        if (size >= 3) {
            for (size_t i = 0; i <= size - 3; i++) {
                if (data[i] == 'b' && data[i+1] == 'u' && data[i+2] == 'g') {
                    printf("Found the demonstration bug! This proves the fuzzer works.\n");
                    printf("Now replace this with real calls to your library functions.\n");
                    // This will be caught by AddressSanitizer
                    int* crash = nullptr;
                    *crash = 42;  // Intentional crash for demo
                }
            }
        }
        
    } catch (const std::exception& e) {
        // Handle exceptions gracefully - they shouldn't crash the fuzzer
    } catch (...) {
        // Catch any other exceptions
    }
{{/if}}
{{/if}}
{{/unless}}
    return 0;
}
