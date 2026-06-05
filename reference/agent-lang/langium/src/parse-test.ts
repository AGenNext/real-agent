import * as fs from 'node:fs';
import { EmptyFileSystem, createDefaultCoreModule, createDefaultSharedCoreModule, inject } from 'langium';
import { AgentLangGeneratedModule, AgentLangGeneratedSharedModule } from './generated/module.js';

const shared = inject(createDefaultSharedCoreModule(EmptyFileSystem), AgentLangGeneratedSharedModule);
const AgentLang = inject(createDefaultCoreModule({ shared }), AgentLangGeneratedModule);
shared.ServiceRegistry.register(AgentLang);

const text = fs.readFileSync(new URL('../../examples/cluster-janitor.agent', import.meta.url), 'utf8');
const result = AgentLang.parser.LangiumParser.parse(text);

const errs = result.lexerErrors.length + result.parserErrors.length;
console.log('lexer errors :', result.lexerErrors.length);
console.log('parser errors:', result.parserErrors.length);
for (const e of result.parserErrors) console.log('  -', e.message);
const ast: any = result.value;
if (errs === 0) {
  console.log(`AST ok: agent=${ast.id} name=${ast.name} version=${ast.version} type=${ast.type}`);
  console.log(`  identity.owner=${ast.identity?.owner} lifecycle=${ast.identity?.lifecycle}`);
  console.log(`  capabilities=${ast.capabilities?.length} actions=${ast.actions?.length}`);
  console.log(`  memory.enable=[${ast.memory?.enable?.join(', ')}] retention=${ast.memory?.retention}`);
  console.log(`  evaluation.minTrust=${ast.evaluation?.minTrust} metrics=${ast.evaluation?.metrics?.length}`);
  console.log('PARSE OK ✓');
}
process.exit(errs ? 1 : 0);
