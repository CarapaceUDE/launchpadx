const fs = require('fs');

// Read and fix ConfigForm.tsx
let content = fs.readFileSync('C:/app/codex-local-launcher/web/src/components/ConfigForm.tsx', 'utf8');

// We need to add newlines at various points
// First, let's insert a newline after the second import statement
content = content.replace(
    /types";\ninterface/,
    'types";\n\ninterface'
);

// After each property definition in the interface (void; followed by a new property)
content = content.replace(/void;\n    models/, 'void;\n\n    models');
content = content.replace(/void;\n    codexInfo/, 'void;\n\n    codexInfo');
content = content.replace(/void;\n    onKillCodex/, 'void;\n\n    onKillCodex');
content = content.replace(/void;\n    onToggleAutoStart/, 'void;\n\n    onToggleAutoStart');

// After the closing brace of the interface
content = content.replace(/}\nexport default function/, '}\n\nexport default function');

// After each const declaration at the top of the function
content = content.replace(/\n    const \[openSections/, '\n\n    const [openSections');
content = content.replace(/\n    const \[ipField/, '\n\n    const [ipField');
content = content.replace(/\n    const hasSynced/, '\n\n    const hasSynced');
content = content.replace(/\n    const \[baseUrlField/, '\n\n    const [baseUrlField');
content = content.replace(/\n    const toggleSection/, '\n\n    const toggleSection');
content = content.replace(/\n    const buildUrl/, '\n\n    const buildUrl');
content = content.replace(/\n    const handleIpChange/, '\n\n    const handleIpChange');
content = content.replace(/\n    const handleBaseUrlChange/, '\n\n    const handleBaseUrlChange');
content = content.replace(/\n    const handlePortChange/, '\n\n    const handlePortChange');
content = content.replace(/\n    const handleSchemeChange/, '\n\n    const handleSchemeChange');
content = content.replace(/\n    const Section/, '\n\n    const Section');
content = content.replace(/\n    const codexIsRunning/, '\n\n    const codexIsRunning');

// After function params, before the opening brace of function body
content = content.replace(/ConfigFormProps\) \{/, 'ConfigFormProps) {');

fs.writeFileSync('C:/app/codex-local-launcher/web/src/components/ConfigForm.tsx', content);
console.log('Done');