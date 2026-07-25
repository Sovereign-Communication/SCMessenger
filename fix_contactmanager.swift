#!/usr/bin/env swift

import Foundation

// Read the generated Swift file
let fileURL = URL(fileURLWithPath: "iOS/SCMessenger/SCMessenger/Generated/api.swift")
var content = try String(contentsOf: fileURL, encoding: .utf8)

// Find the ContactManager class and fix its structure
if let contactManagerRange = content.range(of: "open class ContactManager:", options: [.anchoredSearch]) {
    let startIndex = contactManagerRange.lowerBound
    
    // Find the end of the class (where FfiConverterTypeContactManager starts)
    if let converterStart = content.range(of: "public struct FfiConverterTypeContactManager:", options: [], range: startIndex..<content.endIndex) {
        let endIndex = converterStart.lowerBound
        
        // Extract the ContactManager class section
        let classSection = content[startIndex..<endIndex]
        
        // Find where the class methods start (after deinit)
        if let deinitEnd = classSection.range(of: "try! rustCall { uniffi_scmessenger_core_fn_free_contactmanager(pointer, $0) }", options: []) {
            let methodsStart = deinitEnd.upperBound
            
            // Find the premature closing brace
            if let prematureBrace = classSection[methodsStart..<classSection.endIndex].range(of: "\n\n\n", options: []) {
                let bracePosition = prematureBrace.lowerBound
                
                // Remove the premature closing brace and move methods inside the class
                var fixedSection = String(classSection[..<bracePosition])
                
                // Add proper indentation to all methods
                let methodsPart = String(classSection[bracePosition..<classSection.endIndex])
                let indentedMethods = methodsPart.replacingOccurrations(of: "\nopen func", with: "\n    open func")
                
                fixedSection += indentedMethods
                fixedSection += "\n}\n\n\n"
                
                // Replace the original section with the fixed one
                content.replaceSubrange(startIndex..<endIndex, with: fixedSection)
                
                print("✅ Fixed ContactManager class structure")
            }
        }
    }
}

// Write the fixed content back
try content.write(to: fileURL, atomically: true, encoding: .utf8)
print("✅ Saved fixed Swift file")

catch {
    print("❌ Error: \(error)")
}