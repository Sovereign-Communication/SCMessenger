#!/usr/bin/env bash
# Script to verify platform-specific security configurations
# Requirements: 9.6, 9.11, 9.12 - Verify Android ProGuard, iOS ATS, and no hardcoded passwords

set -euo pipefail

echo " Verifying platform security configurations..."
echo ""

FAILED=0
SECRET_PATTERN="(password|secret|api[-_]?key)[[:space:]]*=[[:space:]]*[\"'][^\"']{8,}[\"']"

# ============================================================================
# Android Security Checks
# ============================================================================
echo " Android Security Checks"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check if Android directory exists
if [[ -d "android" ]]; then
    # Check ProGuard/R8 is enabled for release builds
    if [[ -f "android/app/build.gradle" ]] || [[ -f "android/app/build.gradle.kts" ]]; then
        echo "Checking ProGuard/R8 configuration..."
        
        if grep -Eq "minifyEnabled[[:space:]]*=?[[:space:]]*true" android/app/build.gradle* 2>/dev/null; then
            echo " ProGuard/R8 is enabled for release builds"
        else
            echo " ProGuard/R8 is NOT enabled for release builds"
            echo "   Add to android/app/build.gradle:"
            echo "   buildTypes {"
            echo "       release {"
            echo "           minifyEnabled true"
            echo "           proguardFiles getDefaultProguardFile('proguard-android-optimize.txt'), 'proguard-rules.pro'"
            echo "       }"
            echo "   }"
            FAILED=$((FAILED + 1))
        fi
    else
        echo "️  WARNING: android/app/build.gradle not found"
    fi
    
    # Check for hardcoded secrets in Android code
    echo ""
    echo "Checking for hardcoded secrets in Android code..."
    
    if rg -i "$SECRET_PATTERN" \
        android/app/src --type kotlin --type java 2>/dev/null | grep -v "BuildConfig" | grep -v "test"; then
        echo " Potential hardcoded secrets found in Android code"
        FAILED=$((FAILED + 1))
    else
        echo " No hardcoded secrets found in Android code"
    fi
else
    echo "️  WARNING: android/ directory not found, skipping Android checks"
fi

echo ""

# ============================================================================
# iOS Security Checks
# ============================================================================
echo " iOS Security Checks"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check if iOS directory exists
if [[ -d "iOS" ]]; then
    # Check App Transport Security (ATS) configuration
    IOS_PLIST="iOS/SCMessenger/SCMessenger/Info.plist"
    if [[ -f "$IOS_PLIST" ]]; then
        echo "Checking App Transport Security (ATS) configuration..."
        
        if grep -q "NSAppTransportSecurity" "$IOS_PLIST"; then
            echo " App Transport Security (ATS) is configured"
            
            # Check if ATS is properly configured (not completely disabled)
            if grep -A 5 "NSAppTransportSecurity" "$IOS_PLIST" | grep -q "NSAllowsArbitraryLoads.*true"; then
                echo "️  WARNING: ATS allows arbitrary loads (insecure)"
                echo "   Consider restricting to specific domains with NSExceptionDomains"
            else
                echo " ATS is properly configured (not allowing arbitrary loads)"
            fi
        else
            echo "️  WARNING: App Transport Security (ATS) not explicitly configured"
            echo "   Consider adding NSAppTransportSecurity to Info.plist"
            echo "   Default behavior: HTTPS required for all connections"
        fi
    else
        echo "️  WARNING: iOS/SCMessenger/Info.plist not found"
    fi
    
    # Check for hardcoded secrets in iOS code
    echo ""
    echo "Checking for hardcoded secrets in iOS code..."
    
    if rg -i "$SECRET_PATTERN" \
        iOS/SCMessenger --type swift 2>/dev/null | grep -v "test"; then
        echo " Potential hardcoded secrets found in iOS code"
        FAILED=$((FAILED + 1))
    else
        echo " No hardcoded secrets found in iOS code"
    fi
else
    echo "️  WARNING: iOS/ directory not found, skipping iOS checks"
fi

echo ""

# ============================================================================
# Core Rust Security Checks
# ============================================================================
echo " Core Rust Security Checks"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check for hardcoded secrets in Rust code
echo "Checking for hardcoded secrets in Rust code..."

if rg -i "$SECRET_PATTERN" \
    core/src mobile/src cli/src --type rust 2>/dev/null | \
    grep -v "test" | grep -v "example" | grep -v "TODO"; then
    echo " Potential hardcoded secrets found in Rust code"
    FAILED=$((FAILED + 1))
else
    echo " No hardcoded secrets found in Rust code"
fi

# Check for use of insecure random number generators
echo ""
echo "Checking for insecure random number generators..."

if rg 'use rand::thread_rng' core/src mobile/src --type rust 2>/dev/null | \
    grep -v "test" || true; then
    echo "️  WARNING: thread_rng() found - ensure cryptographic operations use OsRng"
else
    echo " No insecure RNG usage found"
fi

echo ""

# ============================================================================
# Summary
# ============================================================================
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [[ $FAILED -eq 0 ]]; then
    echo " All platform security checks passed"
    exit 0
else
    echo " $FAILED security check(s) failed"
    echo ""
    echo "Please address the issues above before proceeding."
    exit 1
fi
