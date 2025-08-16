#!/usr/bin/env node
/**
 * Generate Mermaid charts from existing Cargo benchmark timing data
 * This script can be used when benchmarks have already been run
 */

const fs = require('fs');
const path = require('path');

function loadTimingDataFromFile() {
    try {
        // Look for Criterion benchmark data in target/criterion
        const criterionPath = 'target/criterion';
        
        if (!fs.existsSync(criterionPath)) {
            console.error("No Criterion benchmark data found. Please run benchmarks first.");
            throw new Error("No Criterion benchmark data found. Please run benchmarks first.");
        }
        
        console.log(`Loading timing data from ${criterionPath}...`);
        
        // Read all benchmark directories
        const implementations = fs.readdirSync(criterionPath)
            .filter(name => !name.startsWith('.') && name !== 'report');
        
        const timingData = [];
        
        for (const impl of implementations) {
            const implPath = path.join(criterionPath, impl);
            const implStats = fs.statSync(implPath);
            
            if (implStats.isDirectory()) {
                const operations = fs.readdirSync(implPath)
                    .filter(name => !name.startsWith('.'));
                
                for (const op of operations) {
                    const opPath = path.join(implPath, op, 'new', 'estimates.json');
                    
                    if (fs.existsSync(opPath)) {
                        try {
                            const content = fs.readFileSync(opPath, 'utf8');
                            const data = JSON.parse(content);
                            
                            // Extract timing data (values are already in nanoseconds)
                            const meanNs = data.mean.point_estimate;
                            const medianNs = data.median.point_estimate;
                            
                            timingData.push({
                                event: 'bench',
                                name: `benchmarks::${impl}::${op}`,
                                implementation: impl,
                                operation: op,
                                mean: meanNs,
                                median: medianNs
                            });
                        } catch (parseError) {
                            console.warn(`Failed to parse ${opPath}: ${parseError.message}`);
                        }
                    }
                }
            }
        }
        
        if (timingData.length > 0) {
            console.log(`Parsed ${timingData.length} benchmark results from Criterion data`);
            return timingData;
        } else {
            console.error("No valid benchmark data found in Criterion output");
            throw new Error("No valid benchmark data found in Criterion output");
        }
    } catch (error) {
        console.error("Error loading timing data:", error.message);
        throw error;
    }
}

function parseCriterionOutput(content) {
    const timingData = [];
    const lines = content.split('\n');
    
    let currentBenchmark = null;
    
    for (let i = 0; i < lines.length; i++) {
        const line = lines[i];
        const trimmedLine = line.trim();
        
        // Look for benchmark name lines (e.g., "QuickBool/first_access")
        if (trimmedLine.includes('/') && !trimmedLine.includes('time:') && !trimmedLine.includes('change:') && !trimmedLine.includes('slope') && !trimmedLine.includes('mean') && !trimmedLine.includes('median')) {
            const match = trimmedLine.match(/^([^/]+)\/(.+)$/);
            if (match) {
                currentBenchmark = {
                    name: `benchmarks::${match[1]}::${match[2]}`,
                    implementation: match[1],
                    operation: match[2]
                };
            }
        }
        
        // Look for timing data lines
        if (currentBenchmark && trimmedLine.startsWith('time:')) {
            const timeMatch = trimmedLine.match(/time:\s*\[([^\]]+)\]/);
            if (timeMatch) {
                const timeStr = timeMatch[1];
                
                // Split the time string into individual time values, keeping numbers and units together
                // The format is like "255.43 ps 256.54 ps 256.82 ps"
                const timeValues = [];
                const parts = timeStr.split(/\s+/).filter(s => s.trim());
                
                for (let j = 0; j < parts.length; j += 2) {
                    if (j + 1 < parts.length) {
                        timeValues.push(parts[j] + ' ' + parts[j + 1]);
                    }
                }
                
                if (timeValues.length >= 3) {
                    // Use the middle value (second value) as the median estimate
                    const medianTime = timeValues[1];
                    const ns = parseTimeToNanoseconds(medianTime);
                    if (ns > 0) {
                        timingData.push({
                            event: 'bench',
                            name: currentBenchmark.name,
                            implementation: currentBenchmark.implementation,
                            operation: currentBenchmark.operation,
                            mean: ns,
                            median: ns
                        });
                    }
                }
            }
        }
        
        // Handle case where benchmark name and time are on the same line
        // Format: "QuickBool/first_access  time:   [1.2073 ns 1.2274 ns 1.2325 ns]"
        if (trimmedLine.includes('/') && trimmedLine.includes('time:')) {
            const nameMatch = trimmedLine.match(/^([^/]+\/[^\s]+)\s+time:\s*\[([^\]]+)\]/);
            if (nameMatch) {
                const nameParts = nameMatch[1].split('/');
                const timeStr = nameMatch[2];
                
                currentBenchmark = {
                    name: `benchmarks::${nameParts[0]}::${nameParts[1]}`,
                    implementation: nameParts[0],
                    operation: nameParts[1]
                };
                
                // Parse the timing data
                const timeValues = [];
                const parts = timeStr.split(/\s+/).filter(s => s.trim());
                
                for (let j = 0; j < parts.length; j += 2) {
                    if (j + 1 < parts.length) {
                        timeValues.push(parts[j] + ' ' + parts[j + 1]);
                    }
                }
                
                if (timeValues.length >= 3) {
                    const medianTime = timeValues[1];
                    const ns = parseTimeToNanoseconds(medianTime);
                    if (ns > 0) {
                        timingData.push({
                            event: 'bench',
                            name: currentBenchmark.name,
                            implementation: currentBenchmark.implementation,
                            operation: currentBenchmark.operation,
                            mean: ns,
                            median: ns
                        });
                    }
                }
            }
        }
    }
    
    return timingData;
}

function parseTimeToNanoseconds(timeStr) {
    const trimmed = timeStr.trim();
    
    // Extract the numeric part and unit
    const match = trimmed.match(/^([0-9.]+)\s*([a-zA-Zµ]+)$/);
    if (!match) {
        return 0;
    }
    
    const numericValue = parseFloat(match[1]);
    const unit = match[2];
    
    if (isNaN(numericValue)) {
        return 0;
    }
    
    // Convert to nanoseconds based on unit
    if (unit === 'ns') {
        return numericValue;
    } else if (unit === 'ps') {
        return numericValue / 1000;
    } else if (unit === 'µs' || unit === 'us') {
        return numericValue * 1000;
    } else if (unit === 'ms') {
        return numericValue * 1000000;
    } else if (unit === 's') {
        return numericValue * 1000000000;
    }
    
    return 0;
}

function parseBenchmarkResults(timingData) {
    if (!timingData || timingData.length === 0) {
        return [];
    }
    
    const results = [];
    
    for (const benchmark of timingData) {
        // Extract implementation and operation from benchmark name
        // Expected formats:
        // 1. "benchmarks::QuickBool::first_access" (individual implementation)
        // 2. "benchmarks::Comparison::QuickBool_first" (comparison - skip these)
        const nameParts = benchmark.name.split('::');
        
        if (nameParts.length >= 3) {
            const groupName = nameParts[1]; // e.g., "QuickBool", "LazyLock", "OnceLock", "Comparison"
            const operation = nameParts[2];  // e.g., "first_access", "cached_access", "QuickBool_first"
            
            // Skip comparison benchmarks as they're redundant
            if (groupName === 'Comparison') {
                continue;
            }
            
            // Extract timing information
            const meanNs = benchmark.mean || 0;
            const medianNs = benchmark.median || 0;
            
            if (meanNs > 0 && medianNs > 0) {
                results.push({
                    implementation: groupName,
                    operation: operation,
                    mean: meanNs,
                    median: medianNs
                });
            }
        }
    }
    
    // Remove duplicates and keep the best (lowest mean) result for each implementation/operation pair
    const uniqueResults = [];
    const seen = new Set();
    
    results.sort((a, b) => a.mean - b.mean);
    
    for (const result of results) {
        const key = `${result.implementation}-${result.operation}`;
        if (!seen.has(key)) {
            seen.add(key);
            uniqueResults.push(result);
        }
    }
    
    return uniqueResults.sort((a, b) => {
        if (a.implementation === b.implementation) {
            return a.operation.localeCompare(b.operation);
        }
        return a.implementation.localeCompare(b.implementation);
    });
}

function generatePerformanceComparisonChart(results) {
    // Filter out operations that don't have meaningful data
    const meaningfulResults = results.filter(r => r.mean > 0.001); // Only include results > 1ps
    
    if (meaningfulResults.length === 0) {
        return '```mermaid\nxychart-beta\n    title "No benchmark data available"\n    x-axis "No data" ["No data"]\n    y-axis "Time" 0 --> 1\n    BAR [0]\n```';
    }
    
    // Only include operations that have data for all implementations
    const implementations = [...new Set(meaningfulResults.map(r => r.implementation))];
    const operationGroups = new Map();
    
    // Group results by operation
    for (const result of meaningfulResults) {
        if (!operationGroups.has(result.operation)) {
            operationGroups.set(result.operation, []);
        }
        operationGroups.get(result.operation).push(result);
    }
    
    // Include operations that have data for at least 2 implementations
    const validOperations = [];
    for (const [operation, results] of operationGroups) {
        if (results.length >= 2) {
            validOperations.push(operation);
        }
    }
    
    if (validOperations.length === 0) {
        return '```mermaid\nxychart-beta\n    title "No benchmark data available"\n    x-axis "No data" ["No data"]\n    y-axis "Time" 0 --> 1\n    BAR [0]\n```';
    }
    
    let chart = '```mermaid\nxychart-beta\n';
    chart += '    title "Performance Comparison: QuickBool vs LazyLock vs OnceLock"\n';
    chart += '    x-axis "Operation" [';
    chart += validOperations.map(op => `"${op}"`).join(', ');
    chart += ']\n';
    
    // Calculate appropriate y-axis range
    const maxValue = Math.max(...meaningfulResults.map(r => r.mean));
    const yMax = Math.ceil(maxValue * 1.2); // Add 20% padding
    
    chart += `    y-axis "Time (nanoseconds)" 0 --> ${yMax}\n`;
    
    // Add data for each implementation
    for (const impl of implementations) {
        const data = validOperations.map(op => {
            const result = meaningfulResults.find(r => 
                r.implementation === impl && r.operation === op);
            if (result && result.mean > 0.001) {
                return result.mean.toFixed(3);
            }
            // If no data for this implementation/operation, skip this implementation
            return null;
        });
        
        // Only include implementations that have data for most operations
        const validData = data.filter(d => d !== null);
        if (validData.length >= validOperations.length * 0.5) { // At least 50% of operations have data
            const cleanData = data.map(d => d || '0.001'); // Use small non-zero value instead of 0
            chart += `    LINE [${cleanData.join(', ')}] "${impl}"\n`;
        }
    }
    
    chart += '```';
    return chart;
}

function generateFirstAccessChart(results) {
    const firstAccess = results.filter(r => r.operation.includes('first') && r.mean > 0.001);
    
    if (firstAccess.length === 0) {
        return '```mermaid\nxychart-beta\n    title "No first access data available"\n    x-axis "No data" ["No data"]\n    y-axis "Time" 0 --> 1\n    BAR [0]\n```';
    }
    
    let chart = '```mermaid\nxychart-beta\n';
    chart += '    title "First Access Performance Comparison"\n';
    chart += '    x-axis "Implementation" [';
    chart += firstAccess.map(r => `"${r.implementation}"`).join(', ');
    chart += ']\n';
    
    // Calculate appropriate y-axis range
    const maxValue = Math.max(...firstAccess.map(r => r.mean));
    const yMax = Math.ceil(maxValue * 1.2); // Add 20% padding
    
    chart += `    y-axis "Time (nanoseconds)" 0 --> ${yMax}\n`;
    
    const data = firstAccess.map(r => r.mean.toFixed(3));
    chart += `    BAR [${data.join(', ')}]\n`;
    
    chart += '```';
    return chart;
}

function generateCachedAccessChart(results) {
    const cachedAccess = results.filter(r => r.operation.includes('cached') && r.mean > 0.001);
    
    if (cachedAccess.length === 0) {
        return '```mermaid\nxychart-beta\n    title "No cached access data available"\n    x-axis "No data" ["No data"]\n    y-axis "Time" 0 --> 1\n    BAR [0]\n```';
    }
    
    // Group by implementation and get the best cached performance
    const bestCached = [];
    const seen = new Set();
    
    for (const result of cachedAccess) {
        if (!seen.has(result.implementation)) {
            seen.add(result.implementation);
            bestCached.push(result);
        }
    }
    
    let chart = '```mermaid\nxychart-beta\n';
            chart += '    title "Cached Access Performance"\n';
    chart += '    x-axis "Implementation" [';
    chart += bestCached.map(r => `"${r.implementation}"`).join(', ');
    chart += ']\n';
    
    // Calculate appropriate y-axis range
    const maxValue = Math.max(...bestCached.map(r => r.mean));
    const yMax = Math.ceil(maxValue * 1.2); // Add 20% padding
    
    chart += `    y-axis "Time (nanoseconds)" 0 --> ${yMax}\n`;
    
    const data = bestCached.map(r => r.mean.toFixed(3));
    chart += `    BAR [${data.join(', ')}]\n`;
    
    chart += '```';
    return chart;
}

// Main execution
function main() {
    console.log("Loading existing benchmark timing data...");
    
    // Load timing data from file
    const timingData = loadTimingDataFromFile();
    
    if (!timingData || timingData.length === 0) {
        console.error("No benchmark timing data found");
        return;
    }
    
    console.log(`Found ${timingData.length} benchmark timing records`);
    
    // Parse the results
    const results = parseBenchmarkResults(timingData);
    
    if (!results || results.length === 0) {
        console.error("No benchmark results found");
        return;
    }
    
    console.log(`Parsed ${results.length} benchmark results`);
    
    // Ensure output directory exists
    const outputDir = "tmp/md";
    fs.mkdirSync(outputDir, { recursive: true });
    
    console.log("Generating Mermaid charts...");
    const charts = {
        firstAccess: generateFirstAccessChart(results),
        cachedAccess: generateCachedAccessChart(results)
    };
    
    // Write charts to files
    fs.writeFileSync('tmp/md/first_access_chart.md', charts.firstAccess);
    fs.writeFileSync('tmp/md/cached_access_chart.md', charts.cachedAccess);
    
    console.log("Mermaid charts generated successfully!");
}

main();
