package dev.jona.tool;

import java.io.IOException;
import java.io.PrintWriter;
import java.lang.invoke.MethodHandles;
import java.nio.charset.StandardCharsets;
import java.util.Arrays;
import java.util.List;

public class GenerateAst {
    public static void main(String[] args) throws IOException {
        if (args.length != 1) {
            System.err.println("Usage: generate_ast <output directory>");
            System.exit(64);
        }
        var outputDir = args[0];
        defineAst(outputDir, "Expr", Arrays.asList(
                "Assign: Token name, Expr value",
                "Binary: Expr left, Token operator, Expr right",
                "Call: Expr callee, Token paren, List<Expr> arguments",
                "Get: Expr object, Token name",
                "Grouping: Expr expression",
                "Literal: Object value",
                "Logical: Expr left, Token operator, Expr right",
                "Set: Expr object, Token name, Expr value",
                "Super: Token keyword, Token method",
                "This: Token keyword",
                "Unary: Token operator, Expr right",
                "Variable: Token name"
        ));
        defineAst(outputDir, "Stmt", Arrays.asList(
                "Block: List<Stmt> statements",
                "Class: Token name, Expr.Variable superclass, List<Stmt.Function> methods",
                "Expression: Expr expression",
                "Function: Token name, List<Token> params, List<Stmt> body",
                "If: Expr condition, Stmt thenBranch, Stmt elseBranch",
                "Print: Expr expression",
                "Return: Token keyword, Expr value",
                "Var: Token name, Expr initializer",
                "While: Expr condition, Stmt body"
        ));
    }

    private static void defineAst(String outputDir, String baseName, List<String> types) throws IOException {
        var path = outputDir + "/" + baseName + ".java";
        var writer = new PrintWriter(path, StandardCharsets.UTF_8);

        // This is way terser than the book's version thanks to records.
        writer.println("package dev.jona.lox;");
        writer.println();
        writer.println("import javax.annotation.processing.Generated;");
        writer.println("import java.util.List;");
        writer.println();
        writer.println("@Generated(\"" + MethodHandles.lookup().lookupClass().getName() + "\")");
        writer.println("interface " + baseName + " {");

        // The base accept() method.
        writer.println("    <R> R accept(Visitor<R> visitor);");

        defineVisitor(writer, baseName, types);

        for (var type : types) {
            var className = type.split(":")[0].trim();
            var fields = type.split(":")[1].trim();
            defineType(writer, baseName, className, fields);
        }

        writer.println("}");
        writer.close();
    }

    private static void defineVisitor(PrintWriter writer, String baseName, List<String> types) {
        writer.println();
        writer.println("    interface Visitor<R> {");

        for (var type : types) {
            var typeName = type.split(":")[0].trim();
            writer.println("        R visit" + typeName + baseName + "(" +
                    typeName + " " + baseName.toLowerCase() + ");");
        }

        writer.println("    }");
    }

    private static void defineType(PrintWriter writer, String baseName, String className, String fieldList) {
        writer.println();
        writer.println("    record " + className + "(" + fieldList + ") implements " + baseName + " {");

        // Visitor pattern.
        writer.println("        @Override");
        writer.println("        public <R> R accept(Visitor<R> visitor) {");
        writer.println("            return visitor.visit" + className + baseName + "(this);");
        writer.println("        }");

        writer.println("    }");
    }
}
