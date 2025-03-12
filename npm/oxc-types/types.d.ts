// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/typescript.rs`

export interface Program extends Span {
  type: 'Program';
  body: Array<Directive | Statement>;
  sourceType: ModuleKind;
  hashbang: Hashbang | null;
}

export type Expression =
  | BooleanLiteral
  | NullLiteral
  | NumericLiteral
  | BigIntLiteral
  | RegExpLiteral
  | StringLiteral
  | TemplateLiteral
  | IdentifierReference
  | MetaProperty
  | Super
  | ArrayExpression
  | ArrowFunctionExpression
  | AssignmentExpression
  | AwaitExpression
  | BinaryExpression
  | CallExpression
  | ChainExpression
  | Class
  | ConditionalExpression
  | Function
  | ImportExpression
  | LogicalExpression
  | NewExpression
  | ObjectExpression
  | ParenthesizedExpression
  | SequenceExpression
  | TaggedTemplateExpression
  | ThisExpression
  | UnaryExpression
  | UpdateExpression
  | YieldExpression
  | PrivateInExpression
  | JSXElement
  | JSXFragment
  | TSAsExpression
  | TSSatisfiesExpression
  | TSTypeAssertion
  | TSNonNullExpression
  | TSInstantiationExpression
  | V8IntrinsicExpression
  | MemberExpression;

export interface IdentifierName extends Span {
  type: 'Identifier';
  name: string;
}

export interface IdentifierReference extends Span {
  type: 'Identifier';
  name: string;
}

export interface BindingIdentifier extends Span {
  type: 'Identifier';
  name: string;
}

export interface LabelIdentifier extends Span {
  type: 'Identifier';
  name: string;
}

export interface ThisExpression extends Span {
  type: 'ThisExpression';
}

export interface ArrayExpression extends Span {
  type: 'ArrayExpression';
  elements: Array<ArrayExpressionElement>;
}

export type ArrayExpressionElement = SpreadElement | null | Expression;

export interface ObjectExpression extends Span {
  type: 'ObjectExpression';
  properties: Array<ObjectPropertyKind>;
}

export type ObjectPropertyKind = ObjectProperty | SpreadElement;

export interface ObjectProperty extends Span {
  type: 'Property';
  method: boolean;
  shorthand: boolean;
  computed: boolean;
  key: PropertyKey;
  value: Expression;
  kind: PropertyKind;
}

export type PropertyKey = IdentifierName | PrivateIdentifier | Expression;

export type PropertyKind = 'init' | 'get' | 'set';

export interface TemplateLiteral extends Span {
  type: 'TemplateLiteral';
  expressions: Array<Expression>;
  quasis: Array<TemplateElement>;
}

export interface TaggedTemplateExpression extends Span {
  type: 'TaggedTemplateExpression';
  tag: Expression;
  quasi: TemplateLiteral;
  typeParameters: TSTypeParameterInstantiation | null;
}

export interface TemplateElement extends Span {
  type: 'TemplateElement';
  value: TemplateElementValue;
  tail: boolean;
}

export interface TemplateElementValue {
  raw: string;
  cooked: string | null;
}

export type MemberExpression = ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression;

export interface ComputedMemberExpression extends Span {
  type: 'MemberExpression';
  object: Expression;
  property: Expression;
  computed: true;
  optional: boolean;
}

export interface StaticMemberExpression extends Span {
  type: 'MemberExpression';
  object: Expression;
  property: IdentifierName;
  computed: false;
  optional: boolean;
}

export interface PrivateFieldExpression extends Span {
  type: 'MemberExpression';
  object: Expression;
  property: PrivateIdentifier;
  computed: false;
  optional: boolean;
}

export interface CallExpression extends Span {
  type: 'CallExpression';
  callee: Expression;
  typeParameters: TSTypeParameterInstantiation | null;
  arguments: Array<Argument>;
  optional: boolean;
}

export interface NewExpression extends Span {
  type: 'NewExpression';
  callee: Expression;
  arguments: Array<Argument>;
  typeParameters: TSTypeParameterInstantiation | null;
}

export interface MetaProperty extends Span {
  type: 'MetaProperty';
  meta: IdentifierName;
  property: IdentifierName;
}

export interface SpreadElement extends Span {
  type: 'SpreadElement';
  argument: Expression;
}

export type Argument = SpreadElement | Expression;

export interface UpdateExpression extends Span {
  type: 'UpdateExpression';
  operator: UpdateOperator;
  prefix: boolean;
  argument: SimpleAssignmentTarget;
}

export interface UnaryExpression extends Span {
  type: 'UnaryExpression';
  operator: UnaryOperator;
  prefix: true;
  argument: Expression;
}

export interface BinaryExpression extends Span {
  type: 'BinaryExpression';
  left: Expression;
  operator: BinaryOperator;
  right: Expression;
}

export interface PrivateInExpression extends Span {
  type: 'BinaryExpression';
  left: PrivateIdentifier;
  operator: 'in';
  right: Expression;
}

export interface LogicalExpression extends Span {
  type: 'LogicalExpression';
  left: Expression;
  operator: LogicalOperator;
  right: Expression;
}

export interface ConditionalExpression extends Span {
  type: 'ConditionalExpression';
  test: Expression;
  consequent: Expression;
  alternate: Expression;
}

export interface AssignmentExpression extends Span {
  type: 'AssignmentExpression';
  operator: AssignmentOperator;
  left: AssignmentTarget;
  right: Expression;
}

export type AssignmentTarget = SimpleAssignmentTarget | AssignmentTargetPattern;

export type SimpleAssignmentTarget =
  | IdentifierReference
  | TSAsExpression
  | TSSatisfiesExpression
  | TSNonNullExpression
  | TSTypeAssertion
  | MemberExpression;

export type AssignmentTargetPattern = ArrayAssignmentTarget | ObjectAssignmentTarget;

export interface ArrayAssignmentTarget extends Span {
  type: 'ArrayPattern';
  elements: Array<AssignmentTargetMaybeDefault | AssignmentTargetRest | null>;
}

export interface ObjectAssignmentTarget extends Span {
  type: 'ObjectPattern';
  properties: Array<AssignmentTargetProperty | AssignmentTargetRest>;
}

export interface AssignmentTargetRest extends Span {
  type: 'RestElement';
  argument: AssignmentTarget;
}

export type AssignmentTargetMaybeDefault = AssignmentTargetWithDefault | AssignmentTarget;

export interface AssignmentTargetWithDefault extends Span {
  type: 'AssignmentPattern';
  left: AssignmentTarget;
  right: Expression;
}

export type AssignmentTargetProperty = AssignmentTargetPropertyIdentifier | AssignmentTargetPropertyProperty;

export interface AssignmentTargetPropertyIdentifier extends Span {
  type: 'Property';
  method: false;
  shorthand: true;
  computed: false;
  key: IdentifierReference;
  value: IdentifierReference | AssignmentTargetWithDefault;
  kind: 'init';
}

export interface AssignmentTargetPropertyProperty extends Span {
  type: 'Property';
  method: false;
  shorthand: false;
  computed: boolean;
  key: PropertyKey;
  value: AssignmentTargetMaybeDefault;
  kind: 'init';
}

export interface SequenceExpression extends Span {
  type: 'SequenceExpression';
  expressions: Array<Expression>;
}

export interface Super extends Span {
  type: 'Super';
}

export interface AwaitExpression extends Span {
  type: 'AwaitExpression';
  argument: Expression;
}

export interface ChainExpression extends Span {
  type: 'ChainExpression';
  expression: ChainElement;
}

export type ChainElement = CallExpression | TSNonNullExpression | MemberExpression;

export interface ParenthesizedExpression extends Span {
  type: 'ParenthesizedExpression';
  expression: Expression;
}

export type Statement =
  | BlockStatement
  | BreakStatement
  | ContinueStatement
  | DebuggerStatement
  | DoWhileStatement
  | EmptyStatement
  | ExpressionStatement
  | ForInStatement
  | ForOfStatement
  | ForStatement
  | IfStatement
  | LabeledStatement
  | ReturnStatement
  | SwitchStatement
  | ThrowStatement
  | TryStatement
  | WhileStatement
  | WithStatement
  | Declaration
  | ModuleDeclaration;

export interface Directive extends Span {
  type: 'ExpressionStatement';
  expression: StringLiteral;
  directive: string;
}

export interface Hashbang extends Span {
  type: 'Hashbang';
  value: string;
}

export interface BlockStatement extends Span {
  type: 'BlockStatement';
  body: Array<Statement>;
}

export type Declaration =
  | VariableDeclaration
  | Function
  | Class
  | TSTypeAliasDeclaration
  | TSInterfaceDeclaration
  | TSEnumDeclaration
  | TSModuleDeclaration
  | TSImportEqualsDeclaration;

export interface VariableDeclaration extends Span {
  type: 'VariableDeclaration';
  declarations: Array<VariableDeclarator>;
  kind: VariableDeclarationKind;
  declare: boolean;
}

export type VariableDeclarationKind = 'var' | 'const' | 'let' | 'using' | 'await using';

export interface VariableDeclarator extends Span {
  type: 'VariableDeclarator';
  id: BindingPattern;
  init: Expression | null;
  definite: boolean;
}

export interface EmptyStatement extends Span {
  type: 'EmptyStatement';
}

export interface ExpressionStatement extends Span {
  type: 'ExpressionStatement';
  expression: Expression;
}

export interface IfStatement extends Span {
  type: 'IfStatement';
  test: Expression;
  consequent: Statement;
  alternate: Statement | null;
}

export interface DoWhileStatement extends Span {
  type: 'DoWhileStatement';
  body: Statement;
  test: Expression;
}

export interface WhileStatement extends Span {
  type: 'WhileStatement';
  test: Expression;
  body: Statement;
}

export interface ForStatement extends Span {
  type: 'ForStatement';
  init: ForStatementInit | null;
  test: Expression | null;
  update: Expression | null;
  body: Statement;
}

export type ForStatementInit = VariableDeclaration | Expression;

export interface ForInStatement extends Span {
  type: 'ForInStatement';
  left: ForStatementLeft;
  right: Expression;
  body: Statement;
}

export type ForStatementLeft = VariableDeclaration | AssignmentTarget;

export interface ForOfStatement extends Span {
  type: 'ForOfStatement';
  await: boolean;
  left: ForStatementLeft;
  right: Expression;
  body: Statement;
}

export interface ContinueStatement extends Span {
  type: 'ContinueStatement';
  label: LabelIdentifier | null;
}

export interface BreakStatement extends Span {
  type: 'BreakStatement';
  label: LabelIdentifier | null;
}

export interface ReturnStatement extends Span {
  type: 'ReturnStatement';
  argument: Expression | null;
}

export interface WithStatement extends Span {
  type: 'WithStatement';
  object: Expression;
  body: Statement;
}

export interface SwitchStatement extends Span {
  type: 'SwitchStatement';
  discriminant: Expression;
  cases: Array<SwitchCase>;
}

export interface SwitchCase extends Span {
  type: 'SwitchCase';
  consequent: Array<Statement>;
  test: Expression | null;
}

export interface LabeledStatement extends Span {
  type: 'LabeledStatement';
  body: Statement;
  label: LabelIdentifier;
}

export interface ThrowStatement extends Span {
  type: 'ThrowStatement';
  argument: Expression;
}

export interface TryStatement extends Span {
  type: 'TryStatement';
  block: BlockStatement;
  handler: CatchClause | null;
  finalizer: BlockStatement | null;
}

export interface CatchClause extends Span {
  type: 'CatchClause';
  param: BindingPattern | null;
  body: BlockStatement;
}

export interface DebuggerStatement extends Span {
  type: 'DebuggerStatement';
}

export type BindingPattern =
  & ({
    typeAnnotation: TSTypeAnnotation | null;
    optional: boolean;
  })
  & (BindingIdentifier | ObjectPattern | ArrayPattern | AssignmentPattern);

export type BindingPatternKind = BindingIdentifier | ObjectPattern | ArrayPattern | AssignmentPattern;

export interface AssignmentPattern extends Span {
  type: 'AssignmentPattern';
  left: BindingPattern;
  right: Expression;
}

export interface ObjectPattern extends Span {
  type: 'ObjectPattern';
  properties: Array<BindingProperty | BindingRestElement>;
}

export interface BindingProperty extends Span {
  type: 'Property';
  method: false;
  shorthand: boolean;
  computed: boolean;
  key: PropertyKey;
  value: BindingPattern;
  kind: 'init';
}

export interface ArrayPattern extends Span {
  type: 'ArrayPattern';
  elements: Array<BindingPattern | BindingRestElement | null>;
}

export interface BindingRestElement extends Span {
  type: 'RestElement';
  argument: BindingPattern;
}

export interface Function extends Span {
  type: FunctionType;
  id: BindingIdentifier | null;
  expression: false;
  generator: boolean;
  async: boolean;
  params: ParamPattern[];
  body: FunctionBody | null;
  declare: boolean;
  typeParameters: TSTypeParameterDeclaration | null;
  thisParam: TSThisParameter | null;
  returnType: TSTypeAnnotation | null;
}

export type ParamPattern = FormalParameter | FormalParameterRest;

export type FunctionType =
  | 'FunctionDeclaration'
  | 'FunctionExpression'
  | 'TSDeclareFunction'
  | 'TSEmptyBodyFunctionExpression';

export interface FormalParameterRest extends Span {
  type: 'RestElement';
  argument: BindingPatternKind;
  typeAnnotation: TSTypeAnnotation | null;
  optional: boolean;
}

export type FormalParameter =
  & ({
    decorators: Array<Decorator>;
    accessibility: TSAccessibility | null;
    readonly: boolean;
    override: boolean;
  })
  & BindingPattern;

export interface FunctionBody extends Span {
  type: 'BlockStatement';
  body: Array<Directive | Statement>;
}

export interface ArrowFunctionExpression extends Span {
  type: 'ArrowFunctionExpression';
  id: null;
  expression: boolean;
  generator: false;
  async: boolean;
  params: ParamPattern[];
  body: FunctionBody | Expression;
  typeParameters: TSTypeParameterDeclaration | null;
  returnType: TSTypeAnnotation | null;
}

export interface YieldExpression extends Span {
  type: 'YieldExpression';
  delegate: boolean;
  argument: Expression | null;
}

export interface Class extends Span {
  type: ClassType;
  id: BindingIdentifier | null;
  superClass: Expression | null;
  body: ClassBody;
  decorators: Array<Decorator>;
  typeParameters: TSTypeParameterDeclaration | null;
  superTypeParameters: TSTypeParameterInstantiation | null;
  implements: Array<TSClassImplements> | null;
  abstract: boolean;
  declare: boolean;
}

export type ClassType = 'ClassDeclaration' | 'ClassExpression';

export interface ClassBody extends Span {
  type: 'ClassBody';
  body: Array<ClassElement>;
}

export type ClassElement = StaticBlock | MethodDefinition | PropertyDefinition | AccessorProperty | TSIndexSignature;

export interface MethodDefinition extends Span {
  type: MethodDefinitionType;
  static: boolean;
  computed: boolean;
  key: PropertyKey;
  kind: MethodDefinitionKind;
  value: Function;
  decorators: Array<Decorator>;
  override: boolean;
  optional: boolean;
  accessibility: TSAccessibility | null;
}

export type MethodDefinitionType = 'MethodDefinition' | 'TSAbstractMethodDefinition';

export interface PropertyDefinition extends Span {
  type: PropertyDefinitionType;
  static: boolean;
  computed: boolean;
  key: PropertyKey;
  value: Expression | null;
  decorators: Array<Decorator>;
  declare: boolean;
  override: boolean;
  optional: boolean;
  definite: boolean;
  readonly: boolean;
  typeAnnotation: TSTypeAnnotation | null;
  accessibility: TSAccessibility | null;
}

export type PropertyDefinitionType = 'PropertyDefinition' | 'TSAbstractPropertyDefinition';

export type MethodDefinitionKind = 'constructor' | 'method' | 'get' | 'set';

export interface PrivateIdentifier extends Span {
  type: 'PrivateIdentifier';
  name: string;
}

export interface StaticBlock extends Span {
  type: 'StaticBlock';
  body: Array<Statement>;
}

export type ModuleDeclaration =
  | ImportDeclaration
  | ExportAllDeclaration
  | ExportDefaultDeclaration
  | ExportNamedDeclaration
  | TSExportAssignment
  | TSNamespaceExportDeclaration;

export type AccessorPropertyType = 'AccessorProperty' | 'TSAbstractAccessorProperty';

export interface AccessorProperty extends Span {
  type: AccessorPropertyType;
  key: PropertyKey;
  value: Expression | null;
  computed: boolean;
  static: boolean;
  decorators: Array<Decorator>;
  definite: boolean;
  typeAnnotation: TSTypeAnnotation | null;
  accessibility: TSAccessibility | null;
}

export interface ImportExpression extends Span {
  type: 'ImportExpression';
  source: Expression;
  options: Expression | null;
}

export interface ImportDeclaration extends Span {
  type: 'ImportDeclaration';
  specifiers: Array<ImportDeclarationSpecifier>;
  source: StringLiteral;
  attributes: Array<ImportAttribute>;
  importKind: ImportOrExportKind;
}

export type ImportPhase = 'source' | 'defer';

export type ImportDeclarationSpecifier = ImportSpecifier | ImportDefaultSpecifier | ImportNamespaceSpecifier;

export interface ImportSpecifier extends Span {
  type: 'ImportSpecifier';
  imported: ModuleExportName;
  local: BindingIdentifier;
  importKind: ImportOrExportKind;
}

export interface ImportDefaultSpecifier extends Span {
  type: 'ImportDefaultSpecifier';
  local: BindingIdentifier;
}

export interface ImportNamespaceSpecifier extends Span {
  type: 'ImportNamespaceSpecifier';
  local: BindingIdentifier;
}

export interface ImportAttribute extends Span {
  type: 'ImportAttribute';
  key: ImportAttributeKey;
  value: StringLiteral;
}

export type ImportAttributeKey = IdentifierName | StringLiteral;

export interface ExportNamedDeclaration extends Span {
  type: 'ExportNamedDeclaration';
  declaration: Declaration | null;
  specifiers: Array<ExportSpecifier>;
  source: StringLiteral | null;
  exportKind: ImportOrExportKind;
  attributes: Array<ImportAttribute>;
}

export interface ExportDefaultDeclaration extends Span {
  type: 'ExportDefaultDeclaration';
  declaration: ExportDefaultDeclarationKind;
}

export interface ExportAllDeclaration extends Span {
  type: 'ExportAllDeclaration';
  exported: ModuleExportName | null;
  source: StringLiteral;
  attributes: Array<ImportAttribute>;
  exportKind: ImportOrExportKind;
}

export interface ExportSpecifier extends Span {
  type: 'ExportSpecifier';
  local: ModuleExportName;
  exported: ModuleExportName;
  exportKind: ImportOrExportKind;
}

export type ExportDefaultDeclarationKind = Function | Class | TSInterfaceDeclaration | Expression;

export type ModuleExportName = IdentifierName | IdentifierReference | StringLiteral;

export interface V8IntrinsicExpression extends Span {
  type: 'V8IntrinsicExpression';
  name: IdentifierName;
  arguments: Array<Argument>;
}

export interface BooleanLiteral extends Span {
  type: 'Literal';
  value: boolean;
  raw: string | null;
}

export interface NullLiteral extends Span {
  type: 'Literal';
  value: null;
  raw: 'null' | null;
}

export interface NumericLiteral extends Span {
  type: 'Literal';
  value: number;
  raw: string | null;
}

export interface StringLiteral extends Span {
  type: 'Literal';
  value: string;
  raw: string | null;
}

export interface BigIntLiteral extends Span {
  type: 'Literal';
  value: BigInt;
  raw: string | null;
  bigint: string;
}

export interface RegExpLiteral extends Span {
  type: 'Literal';
  value: RegExp | null;
  raw: string | null;
  regex: RegExp;
}

export interface RegExp {
  pattern: string;
  flags: string;
}

export interface JSXElement extends Span {
  type: 'JSXElement';
  openingElement: JSXOpeningElement;
  closingElement: JSXClosingElement | null;
  children: Array<JSXChild>;
}

export interface JSXOpeningElement extends Span {
  type: 'JSXOpeningElement';
  selfClosing: boolean;
  name: JSXElementName;
  attributes: Array<JSXAttributeItem>;
  typeParameters: TSTypeParameterInstantiation | null;
}

export interface JSXClosingElement extends Span {
  type: 'JSXClosingElement';
  name: JSXElementName;
}

export interface JSXFragment extends Span {
  type: 'JSXFragment';
  openingFragment: JSXOpeningFragment;
  closingFragment: JSXClosingFragment;
  children: Array<JSXChild>;
}

export interface JSXOpeningFragment extends Span {
  type: 'JSXOpeningFragment';
}

export interface JSXClosingFragment extends Span {
  type: 'JSXClosingFragment';
}

export type JSXElementName = JSXIdentifier | JSXNamespacedName | JSXMemberExpression;

export interface JSXNamespacedName extends Span {
  type: 'JSXNamespacedName';
  namespace: JSXIdentifier;
  name: JSXIdentifier;
}

export interface JSXMemberExpression extends Span {
  type: 'JSXMemberExpression';
  object: JSXMemberExpressionObject;
  property: JSXIdentifier;
}

export type JSXMemberExpressionObject = JSXIdentifier | JSXMemberExpression;

export interface JSXExpressionContainer extends Span {
  type: 'JSXExpressionContainer';
  expression: JSXExpression;
}

export type JSXExpression = JSXEmptyExpression | Expression;

export interface JSXEmptyExpression extends Span {
  type: 'JSXEmptyExpression';
}

export type JSXAttributeItem = JSXAttribute | JSXSpreadAttribute;

export interface JSXAttribute extends Span {
  type: 'JSXAttribute';
  name: JSXAttributeName;
  value: JSXAttributeValue | null;
}

export interface JSXSpreadAttribute extends Span {
  type: 'JSXSpreadAttribute';
  argument: Expression;
}

export type JSXAttributeName = JSXIdentifier | JSXNamespacedName;

export type JSXAttributeValue = StringLiteral | JSXExpressionContainer | JSXElement | JSXFragment;

export interface JSXIdentifier extends Span {
  type: 'JSXIdentifier';
  name: string;
}

export type JSXChild = JSXText | JSXElement | JSXFragment | JSXExpressionContainer | JSXSpreadChild;

export interface JSXSpreadChild extends Span {
  type: 'JSXSpreadChild';
  expression: Expression;
}

export interface JSXText extends Span {
  type: 'JSXText';
  value: string;
}

export interface TSThisParameter extends Span {
  type: 'TSThisParameter';
  typeAnnotation: TSTypeAnnotation | null;
}

export interface TSEnumDeclaration extends Span {
  type: 'TSEnumDeclaration';
  id: BindingIdentifier;
  members: Array<TSEnumMember>;
  const: boolean;
  declare: boolean;
}

export interface TSEnumMember extends Span {
  type: 'TSEnumMember';
  id: TSEnumMemberName;
  initializer: Expression | null;
}

export type TSEnumMemberName = IdentifierName | StringLiteral;

export interface TSTypeAnnotation extends Span {
  type: 'TSTypeAnnotation';
  typeAnnotation: TSType;
}

export interface TSLiteralType extends Span {
  type: 'TSLiteralType';
  literal: TSLiteral;
}

export type TSLiteral =
  | BooleanLiteral
  | NumericLiteral
  | BigIntLiteral
  | StringLiteral
  | TemplateLiteral
  | UnaryExpression;

export type TSType =
  | TSAnyKeyword
  | TSBigIntKeyword
  | TSBooleanKeyword
  | TSIntrinsicKeyword
  | TSNeverKeyword
  | TSNullKeyword
  | TSNumberKeyword
  | TSObjectKeyword
  | TSStringKeyword
  | TSSymbolKeyword
  | TSUndefinedKeyword
  | TSUnknownKeyword
  | TSVoidKeyword
  | TSArrayType
  | TSConditionalType
  | TSConstructorType
  | TSFunctionType
  | TSImportType
  | TSIndexedAccessType
  | TSInferType
  | TSIntersectionType
  | TSLiteralType
  | TSMappedType
  | TSNamedTupleMember
  | TSTemplateLiteralType
  | TSThisType
  | TSTupleType
  | TSTypeLiteral
  | TSTypeOperator
  | TSTypePredicate
  | TSTypeQuery
  | TSTypeReference
  | TSUnionType
  | TSParenthesizedType
  | JSDocNullableType
  | JSDocNonNullableType
  | JSDocUnknownType;

export interface TSConditionalType extends Span {
  type: 'TSConditionalType';
  checkType: TSType;
  extendsType: TSType;
  trueType: TSType;
  falseType: TSType;
}

export interface TSUnionType extends Span {
  type: 'TSUnionType';
  types: Array<TSType>;
}

export interface TSIntersectionType extends Span {
  type: 'TSIntersectionType';
  types: Array<TSType>;
}

export interface TSParenthesizedType extends Span {
  type: 'TSParenthesizedType';
  typeAnnotation: TSType;
}

export interface TSTypeOperator extends Span {
  type: 'TSTypeOperator';
  operator: TSTypeOperatorOperator;
  typeAnnotation: TSType;
}

export type TSTypeOperatorOperator = 'keyof' | 'unique' | 'readonly';

export interface TSArrayType extends Span {
  type: 'TSArrayType';
  elementType: TSType;
}

export interface TSIndexedAccessType extends Span {
  type: 'TSIndexedAccessType';
  objectType: TSType;
  indexType: TSType;
}

export interface TSTupleType extends Span {
  type: 'TSTupleType';
  elementTypes: Array<TSTupleElement>;
}

export interface TSNamedTupleMember extends Span {
  type: 'TSNamedTupleMember';
  elementType: TSTupleElement;
  label: IdentifierName;
  optional: boolean;
}

export interface TSOptionalType extends Span {
  type: 'TSOptionalType';
  typeAnnotation: TSType;
}

export interface TSRestType extends Span {
  type: 'TSRestType';
  typeAnnotation: TSType;
}

export type TSTupleElement = TSOptionalType | TSRestType | TSType;

export interface TSAnyKeyword extends Span {
  type: 'TSAnyKeyword';
}

export interface TSStringKeyword extends Span {
  type: 'TSStringKeyword';
}

export interface TSBooleanKeyword extends Span {
  type: 'TSBooleanKeyword';
}

export interface TSNumberKeyword extends Span {
  type: 'TSNumberKeyword';
}

export interface TSNeverKeyword extends Span {
  type: 'TSNeverKeyword';
}

export interface TSIntrinsicKeyword extends Span {
  type: 'TSIntrinsicKeyword';
}

export interface TSUnknownKeyword extends Span {
  type: 'TSUnknownKeyword';
}

export interface TSNullKeyword extends Span {
  type: 'TSNullKeyword';
}

export interface TSUndefinedKeyword extends Span {
  type: 'TSUndefinedKeyword';
}

export interface TSVoidKeyword extends Span {
  type: 'TSVoidKeyword';
}

export interface TSSymbolKeyword extends Span {
  type: 'TSSymbolKeyword';
}

export interface TSThisType extends Span {
  type: 'TSThisType';
}

export interface TSObjectKeyword extends Span {
  type: 'TSObjectKeyword';
}

export interface TSBigIntKeyword extends Span {
  type: 'TSBigIntKeyword';
}

export interface TSTypeReference extends Span {
  type: 'TSTypeReference';
  typeName: TSTypeName;
  typeParameters: TSTypeParameterInstantiation | null;
}

export type TSTypeName = IdentifierReference | TSQualifiedName;

export interface TSQualifiedName extends Span {
  type: 'TSQualifiedName';
  left: TSTypeName;
  right: IdentifierName;
}

export interface TSTypeParameterInstantiation extends Span {
  type: 'TSTypeParameterInstantiation';
  params: Array<TSType>;
}

export interface TSTypeParameter extends Span {
  type: 'TSTypeParameter';
  name: BindingIdentifier;
  constraint: TSType | null;
  default: TSType | null;
  in: boolean;
  out: boolean;
  const: boolean;
}

export interface TSTypeParameterDeclaration extends Span {
  type: 'TSTypeParameterDeclaration';
  params: Array<TSTypeParameter>;
}

export interface TSTypeAliasDeclaration extends Span {
  type: 'TSTypeAliasDeclaration';
  id: BindingIdentifier;
  typeParameters: TSTypeParameterDeclaration | null;
  typeAnnotation: TSType;
  declare: boolean;
}

export type TSAccessibility = 'private' | 'protected' | 'public';

export interface TSClassImplements extends Span {
  type: 'TSClassImplements';
  expression: TSTypeName;
  typeParameters: TSTypeParameterInstantiation | null;
}

export interface TSInterfaceDeclaration extends Span {
  type: 'TSInterfaceDeclaration';
  id: BindingIdentifier;
  extends: Array<TSInterfaceHeritage> | null;
  typeParameters: TSTypeParameterDeclaration | null;
  body: TSInterfaceBody;
  declare: boolean;
}

export interface TSInterfaceBody extends Span {
  type: 'TSInterfaceBody';
  body: Array<TSSignature>;
}

export interface TSPropertySignature extends Span {
  type: 'TSPropertySignature';
  computed: boolean;
  optional: boolean;
  readonly: boolean;
  key: PropertyKey;
  typeAnnotation: TSTypeAnnotation | null;
}

export type TSSignature =
  | TSIndexSignature
  | TSPropertySignature
  | TSCallSignatureDeclaration
  | TSConstructSignatureDeclaration
  | TSMethodSignature;

export interface TSIndexSignature extends Span {
  type: 'TSIndexSignature';
  parameters: Array<TSIndexSignatureName>;
  typeAnnotation: TSTypeAnnotation;
  readonly: boolean;
  static: boolean;
}

export interface TSCallSignatureDeclaration extends Span {
  type: 'TSCallSignatureDeclaration';
  typeParameters: TSTypeParameterDeclaration | null;
  thisParam: TSThisParameter | null;
  params: ParamPattern[];
  returnType: TSTypeAnnotation | null;
}

export type TSMethodSignatureKind = 'method' | 'get' | 'set';

export interface TSMethodSignature extends Span {
  type: 'TSMethodSignature';
  key: PropertyKey;
  computed: boolean;
  optional: boolean;
  kind: TSMethodSignatureKind;
  typeParameters: TSTypeParameterDeclaration | null;
  thisParam: TSThisParameter | null;
  params: ParamPattern[];
  returnType: TSTypeAnnotation | null;
}

export interface TSConstructSignatureDeclaration extends Span {
  type: 'TSConstructSignatureDeclaration';
  typeParameters: TSTypeParameterDeclaration | null;
  params: ParamPattern[];
  returnType: TSTypeAnnotation | null;
}

export interface TSIndexSignatureName extends Span {
  type: 'Identifier';
  name: string;
  typeAnnotation: TSTypeAnnotation;
}

export interface TSInterfaceHeritage extends Span {
  type: 'TSInterfaceHeritage';
  expression: Expression;
  typeParameters: TSTypeParameterInstantiation | null;
}

export interface TSTypePredicate extends Span {
  type: 'TSTypePredicate';
  parameterName: TSTypePredicateName;
  asserts: boolean;
  typeAnnotation: TSTypeAnnotation | null;
}

export type TSTypePredicateName = IdentifierName | TSThisType;

export interface TSModuleDeclaration extends Span {
  type: 'TSModuleDeclaration';
  id: TSModuleDeclarationName;
  body: TSModuleDeclarationBody | null;
  kind: TSModuleDeclarationKind;
  declare: boolean;
}

export type TSModuleDeclarationKind = 'global' | 'module' | 'namespace';

export type TSModuleDeclarationName = BindingIdentifier | StringLiteral;

export type TSModuleDeclarationBody = TSModuleDeclaration | TSModuleBlock;

export interface TSModuleBlock extends Span {
  type: 'TSModuleBlock';
  body: Array<Directive | Statement>;
}

export interface TSTypeLiteral extends Span {
  type: 'TSTypeLiteral';
  members: Array<TSSignature>;
}

export interface TSInferType extends Span {
  type: 'TSInferType';
  typeParameter: TSTypeParameter;
}

export interface TSTypeQuery extends Span {
  type: 'TSTypeQuery';
  exprName: TSTypeQueryExprName;
  typeParameters: TSTypeParameterInstantiation | null;
}

export type TSTypeQueryExprName = TSImportType | TSTypeName;

export interface TSImportType extends Span {
  type: 'TSImportType';
  argument: TSType;
  options: TSImportAttributes | null;
  qualifier: TSTypeName | null;
  typeArguments: TSTypeParameterInstantiation | null;
  isTypeOf: boolean;
}

export interface TSImportAttributes extends Span {
  type: 'TSImportAttributes';
  attributesKeyword: IdentifierName;
  elements: Array<TSImportAttribute>;
}

export interface TSImportAttribute extends Span {
  type: 'TSImportAttribute';
  name: TSImportAttributeName;
  value: Expression;
}

export type TSImportAttributeName = IdentifierName | StringLiteral;

export interface TSFunctionType extends Span {
  type: 'TSFunctionType';
  typeParameters: TSTypeParameterDeclaration | null;
  thisParam: TSThisParameter | null;
  params: ParamPattern[];
  returnType: TSTypeAnnotation;
}

export interface TSConstructorType extends Span {
  type: 'TSConstructorType';
  abstract: boolean;
  typeParameters: TSTypeParameterDeclaration | null;
  params: ParamPattern[];
  returnType: TSTypeAnnotation;
}

export interface TSMappedType extends Span {
  type: 'TSMappedType';
  typeParameter: TSTypeParameter;
  nameType: TSType | null;
  typeAnnotation: TSType | null;
  optional: TSMappedTypeModifierOperator;
  readonly: TSMappedTypeModifierOperator;
}

export type TSMappedTypeModifierOperator = 'true' | '+' | '-' | 'none';

export interface TSTemplateLiteralType extends Span {
  type: 'TSTemplateLiteralType';
  quasis: Array<TemplateElement>;
  types: Array<TSType>;
}

export interface TSAsExpression extends Span {
  type: 'TSAsExpression';
  expression: Expression;
  typeAnnotation: TSType;
}

export interface TSSatisfiesExpression extends Span {
  type: 'TSSatisfiesExpression';
  expression: Expression;
  typeAnnotation: TSType;
}

export interface TSTypeAssertion extends Span {
  type: 'TSTypeAssertion';
  expression: Expression;
  typeAnnotation: TSType;
}

export interface TSImportEqualsDeclaration extends Span {
  type: 'TSImportEqualsDeclaration';
  id: BindingIdentifier;
  moduleReference: TSModuleReference;
  importKind: ImportOrExportKind;
}

export type TSModuleReference = TSExternalModuleReference | TSTypeName;

export interface TSExternalModuleReference extends Span {
  type: 'TSExternalModuleReference';
  expression: StringLiteral;
}

export interface TSNonNullExpression extends Span {
  type: 'TSNonNullExpression';
  expression: Expression;
}

export interface Decorator extends Span {
  type: 'Decorator';
  expression: Expression;
}

export interface TSExportAssignment extends Span {
  type: 'TSExportAssignment';
  expression: Expression;
}

export interface TSNamespaceExportDeclaration extends Span {
  type: 'TSNamespaceExportDeclaration';
  id: IdentifierName;
}

export interface TSInstantiationExpression extends Span {
  type: 'TSInstantiationExpression';
  expression: Expression;
  typeParameters: TSTypeParameterInstantiation;
}

export type ImportOrExportKind = 'value' | 'type';

export interface JSDocNullableType extends Span {
  type: 'JSDocNullableType';
  typeAnnotation: TSType;
  postfix: boolean;
}

export interface JSDocNonNullableType extends Span {
  type: 'JSDocNonNullableType';
  typeAnnotation: TSType;
  postfix: boolean;
}

export interface JSDocUnknownType extends Span {
  type: 'JSDocUnknownType';
}

export type AssignmentOperator =
  | '='
  | '+='
  | '-='
  | '*='
  | '/='
  | '%='
  | '**='
  | '<<='
  | '>>='
  | '>>>='
  | '|='
  | '^='
  | '&='
  | '||='
  | '&&='
  | '??=';

export type BinaryOperator =
  | '=='
  | '!='
  | '==='
  | '!=='
  | '<'
  | '<='
  | '>'
  | '>='
  | '+'
  | '-'
  | '*'
  | '/'
  | '%'
  | '**'
  | '<<'
  | '>>'
  | '>>>'
  | '|'
  | '^'
  | '&'
  | 'in'
  | 'instanceof';

export type LogicalOperator = '||' | '&&' | '??';

export type UnaryOperator = '+' | '-' | '!' | '~' | 'typeof' | 'void' | 'delete';

export type UpdateOperator = '++' | '--';

export interface Span {
  start: number;
  end: number;
}

export type ModuleKind = 'script' | 'module' | 'unambiguous';

export interface Pattern extends Span {
  type: 'Pattern';
  body: Disjunction;
}

export interface Disjunction extends Span {
  type: 'Disjunction';
  body: Array<Alternative>;
}

export interface Alternative extends Span {
  type: 'Alternative';
  body: Array<Term>;
}

export type Term =
  | BoundaryAssertion
  | LookAroundAssertion
  | Quantifier
  | Character
  | Dot
  | CharacterClassEscape
  | UnicodePropertyEscape
  | CharacterClass
  | CapturingGroup
  | IgnoreGroup
  | IndexedReference
  | NamedReference;

export interface BoundaryAssertion extends Span {
  type: 'BoundaryAssertion';
  kind: BoundaryAssertionKind;
}

export type BoundaryAssertionKind = 'start' | 'end' | 'boundary' | 'negativeBoundary';

export interface LookAroundAssertion extends Span {
  type: 'LookAroundAssertion';
  kind: LookAroundAssertionKind;
  body: Disjunction;
}

export type LookAroundAssertionKind = 'lookahead' | 'negativeLookahead' | 'lookbehind' | 'negativeLookbehind';

export interface Quantifier extends Span {
  type: 'Quantifier';
  min: number;
  max: number | null;
  greedy: boolean;
  body: Term;
}

export interface Character extends Span {
  type: 'Character';
  kind: CharacterKind;
  value: number;
}

export type CharacterKind =
  | 'controlLetter'
  | 'hexadecimalEscape'
  | 'identifier'
  | 'null'
  | 'octal1'
  | 'octal2'
  | 'octal3'
  | 'singleEscape'
  | 'symbol'
  | 'unicodeEscape';

export interface CharacterClassEscape extends Span {
  type: 'CharacterClassEscape';
  kind: CharacterClassEscapeKind;
}

export type CharacterClassEscapeKind = 'd' | 'negativeD' | 's' | 'negativeS' | 'w' | 'negativeW';

export interface UnicodePropertyEscape extends Span {
  type: 'UnicodePropertyEscape';
  negative: boolean;
  strings: boolean;
  name: string;
  value: string | null;
}

export interface Dot extends Span {
  type: 'Dot';
}

export interface CharacterClass extends Span {
  type: 'CharacterClass';
  negative: boolean;
  strings: boolean;
  kind: CharacterClassContentsKind;
  body: Array<CharacterClassContents>;
}

export type CharacterClassContentsKind = 'union' | 'intersection' | 'subtraction';

export type CharacterClassContents =
  | CharacterClassRange
  | CharacterClassEscape
  | UnicodePropertyEscape
  | Character
  | CharacterClass
  | ClassStringDisjunction;

export interface CharacterClassRange extends Span {
  type: 'CharacterClassRange';
  min: Character;
  max: Character;
}

export interface ClassStringDisjunction extends Span {
  type: 'ClassStringDisjunction';
  strings: boolean;
  body: Array<ClassString>;
}

export interface ClassString extends Span {
  type: 'ClassString';
  strings: boolean;
  body: Array<Character>;
}

export interface CapturingGroup extends Span {
  type: 'CapturingGroup';
  name: string | null;
  body: Disjunction;
}

export interface IgnoreGroup extends Span {
  type: 'IgnoreGroup';
  modifiers: Modifiers | null;
  body: Disjunction;
}

export interface Modifiers extends Span {
  type: 'Modifiers';
  enabling: Modifier | null;
  disabling: Modifier | null;
}

export interface Modifier {
  type: 'Modifier';
  ignoreCase: boolean;
  multiline: boolean;
  sticky: boolean;
}

export interface IndexedReference extends Span {
  type: 'IndexedReference';
  index: number;
}

export interface NamedReference extends Span {
  type: 'NamedReference';
  name: string;
}

export type Node =
  | Program
  | IdentifierName
  | IdentifierReference
  | BindingIdentifier
  | LabelIdentifier
  | ThisExpression
  | ArrayExpression
  | ObjectExpression
  | ObjectProperty
  | TemplateLiteral
  | TaggedTemplateExpression
  | TemplateElement
  | ComputedMemberExpression
  | StaticMemberExpression
  | PrivateFieldExpression
  | CallExpression
  | NewExpression
  | MetaProperty
  | SpreadElement
  | UpdateExpression
  | UnaryExpression
  | BinaryExpression
  | PrivateInExpression
  | LogicalExpression
  | ConditionalExpression
  | AssignmentExpression
  | ArrayAssignmentTarget
  | ObjectAssignmentTarget
  | AssignmentTargetRest
  | AssignmentTargetWithDefault
  | AssignmentTargetPropertyIdentifier
  | AssignmentTargetPropertyProperty
  | SequenceExpression
  | Super
  | AwaitExpression
  | ChainExpression
  | ParenthesizedExpression
  | Directive
  | Hashbang
  | BlockStatement
  | VariableDeclaration
  | VariableDeclarator
  | EmptyStatement
  | ExpressionStatement
  | IfStatement
  | DoWhileStatement
  | WhileStatement
  | ForStatement
  | ForInStatement
  | ForOfStatement
  | ContinueStatement
  | BreakStatement
  | ReturnStatement
  | WithStatement
  | SwitchStatement
  | SwitchCase
  | LabeledStatement
  | ThrowStatement
  | TryStatement
  | CatchClause
  | DebuggerStatement
  | AssignmentPattern
  | ObjectPattern
  | BindingProperty
  | ArrayPattern
  | BindingRestElement
  | Function
  | FunctionBody
  | ArrowFunctionExpression
  | YieldExpression
  | Class
  | ClassBody
  | MethodDefinition
  | PropertyDefinition
  | PrivateIdentifier
  | StaticBlock
  | AccessorProperty
  | ImportExpression
  | ImportDeclaration
  | ImportSpecifier
  | ImportDefaultSpecifier
  | ImportNamespaceSpecifier
  | ImportAttribute
  | ExportNamedDeclaration
  | ExportDefaultDeclaration
  | ExportAllDeclaration
  | ExportSpecifier
  | V8IntrinsicExpression
  | BooleanLiteral
  | NullLiteral
  | NumericLiteral
  | StringLiteral
  | BigIntLiteral
  | RegExpLiteral
  | JSXElement
  | JSXOpeningElement
  | JSXClosingElement
  | JSXFragment
  | JSXOpeningFragment
  | JSXClosingFragment
  | JSXNamespacedName
  | JSXMemberExpression
  | JSXExpressionContainer
  | JSXEmptyExpression
  | JSXAttribute
  | JSXSpreadAttribute
  | JSXIdentifier
  | JSXSpreadChild
  | JSXText
  | TSThisParameter
  | TSEnumDeclaration
  | TSEnumMember
  | TSTypeAnnotation
  | TSLiteralType
  | TSConditionalType
  | TSUnionType
  | TSIntersectionType
  | TSParenthesizedType
  | TSTypeOperator
  | TSArrayType
  | TSIndexedAccessType
  | TSTupleType
  | TSNamedTupleMember
  | TSOptionalType
  | TSRestType
  | TSAnyKeyword
  | TSStringKeyword
  | TSBooleanKeyword
  | TSNumberKeyword
  | TSNeverKeyword
  | TSIntrinsicKeyword
  | TSUnknownKeyword
  | TSNullKeyword
  | TSUndefinedKeyword
  | TSVoidKeyword
  | TSSymbolKeyword
  | TSThisType
  | TSObjectKeyword
  | TSBigIntKeyword
  | TSTypeReference
  | TSQualifiedName
  | TSTypeParameterInstantiation
  | TSTypeParameter
  | TSTypeParameterDeclaration
  | TSTypeAliasDeclaration
  | TSClassImplements
  | TSInterfaceDeclaration
  | TSInterfaceBody
  | TSPropertySignature
  | TSIndexSignature
  | TSCallSignatureDeclaration
  | TSMethodSignature
  | TSConstructSignatureDeclaration
  | TSIndexSignatureName
  | TSInterfaceHeritage
  | TSTypePredicate
  | TSModuleDeclaration
  | TSModuleBlock
  | TSTypeLiteral
  | TSInferType
  | TSTypeQuery
  | TSImportType
  | TSImportAttributes
  | TSImportAttribute
  | TSFunctionType
  | TSConstructorType
  | TSMappedType
  | TSTemplateLiteralType
  | TSAsExpression
  | TSSatisfiesExpression
  | TSTypeAssertion
  | TSImportEqualsDeclaration
  | TSExternalModuleReference
  | TSNonNullExpression
  | Decorator
  | TSExportAssignment
  | TSNamespaceExportDeclaration
  | TSInstantiationExpression
  | JSDocNullableType
  | JSDocNonNullableType
  | JSDocUnknownType
  | Pattern
  | Disjunction
  | Alternative
  | BoundaryAssertion
  | LookAroundAssertion
  | Quantifier
  | Character
  | CharacterClassEscape
  | UnicodePropertyEscape
  | Dot
  | CharacterClass
  | CharacterClassRange
  | ClassStringDisjunction
  | ClassString
  | CapturingGroup
  | IgnoreGroup
  | Modifiers
  | Modifier
  | IndexedReference
  | NamedReference
  | FormalParameterRest;
