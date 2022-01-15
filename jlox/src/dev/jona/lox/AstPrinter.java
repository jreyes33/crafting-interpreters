package dev.jona.lox;

import java.util.List;

public class AstPrinter implements Expr.Visitor<String>, Stmt.Visitor<String> {
    // TODO: this method is only used for testing; delete it.
    public static void main(String[] args) {
        var expression = new Expr.Binary(
                new Expr.Unary(
                        new Token(TokenType.MINUS, "-", null, 1),
                        new Expr.Literal(123)),
                new Token(TokenType.STAR, "*", null, 1),
                new Expr.Grouping(new Expr.Literal(45.67)));

        System.out.println(new AstPrinter().print(expression));
    }

    String print(Expr expr) {
        return expr.accept(this);
    }

    String print(Stmt stmt) {
        return stmt.accept(this);
    }

    @Override
    public String visitAssignExpr(Expr.Assign expr) {
        return parenthesize2("=", expr.name().lexeme(), expr.value());
    }

    @Override
    public String visitBinaryExpr(Expr.Binary expr) {
        return parenthesize(expr.operator().lexeme(), expr.left(), expr.right());
    }

    @Override
    public String visitCallExpr(Expr.Call expr) {
        return parenthesize2("call", expr.callee(), expr.arguments());
    }

    @Override
    public String visitGetExpr(Expr.Get expr) {
        return parenthesize2(".", expr.object(), expr.name().lexeme());
    }

    @Override
    public String visitGroupingExpr(Expr.Grouping expr) {
        return parenthesize("group", expr.expression());
    }

    @Override
    public String visitLiteralExpr(Expr.Literal expr) {
        if (expr.value() == null) return "nil";
        return expr.value().toString();
    }

    @Override
    public String visitLogicalExpr(Expr.Logical expr) {
        return parenthesize(expr.operator().lexeme(), expr.left(), expr.right());
    }

    @Override
    public String visitSetExpr(Expr.Set expr) {
        return parenthesize2("=", expr.object(), expr.name().lexeme(), expr.value());
    }

    @Override
    public String visitSuperExpr(Expr.Super expr) {
        return parenthesize2("super", expr.method());
    }

    @Override
    public String visitThisExpr(Expr.This expr) {
        return "this";
    }

    @Override
    public String visitUnaryExpr(Expr.Unary expr) {
        return parenthesize(expr.operator().lexeme(), expr.right());
    }

    @Override
    public String visitVariableExpr(Expr.Variable expr) {
        return expr.name().lexeme();
    }

    @Override
    public String visitBlockStmt(Stmt.Block stmt) {
        var builder = new StringBuilder();
        builder.append("(block ");

        for (var statement : stmt.statements()) {
            builder.append(statement.accept(this));
        }

        builder.append(")");
        return builder.toString();
    }

    @Override
    public String visitClassStmt(Stmt.Class stmt) {
        var builder = new StringBuilder();
        builder.append("(class ").append(stmt.name().lexeme());

        if (stmt.superclass() != null) {
            builder.append(" < ").append(print(stmt.superclass()));
        }

        for (var method : stmt.methods()) {
            builder.append(" ").append(print(method));
        }

        builder.append(")");
        return builder.toString();
    }

    @Override
    public String visitExpressionStmt(Stmt.Expression stmt) {
        return parenthesize(";", stmt.expression());
    }

    @Override
    public String visitFunctionStmt(Stmt.Function stmt) {
        var builder = new StringBuilder();
        builder.append("(fun ").append(stmt.name().lexeme()).append("(");

        for (var param : stmt.params()) {
            if (param != stmt.params().get(0)) builder.append(" ");
            builder.append(param.lexeme());
        }

        builder.append(") ");

        for (var body : stmt.body()) {
            builder.append(body.accept(this));
        }

        builder.append(")");
        return builder.toString();
    }

    @Override
    public String visitIfStmt(Stmt.If stmt) {
        if (stmt.elseBranch() == null) {
            return parenthesize2("if", stmt.condition(), stmt.thenBranch());
        }

        return parenthesize2("if", stmt.condition(), stmt.thenBranch(), stmt.elseBranch());
    }

    @Override
    public String visitPrintStmt(Stmt.Print stmt) {
        return parenthesize("print", stmt.expression());
    }

    @Override
    public String visitReturnStmt(Stmt.Return stmt) {
        if (stmt.value() == null) return "(return)";
        return parenthesize("return", stmt.value());
    }

    @Override
    public String visitVarStmt(Stmt.Var stmt) {
        if (stmt.initializer() == null) {
            return parenthesize2("var", stmt.name());
        }

        return parenthesize2("var", stmt.name(), "=", stmt.initializer());
    }

    @Override
    public String visitWhileStmt(Stmt.While stmt) {
        return parenthesize2("while", stmt.condition(), stmt.body());
    }

    private String parenthesize(String name, Expr... exprs) {
        var builder = new StringBuilder();

        builder.append("(").append(name);
        for (var expr : exprs) {
            builder.append(" ");
            builder.append(expr.accept(this));
        }
        builder.append(")");

        return builder.toString();
    }

    private String parenthesize2(String name, Object... parts) {
        var builder = new StringBuilder();

        builder.append("(").append(name);
        transform(builder, parts);
        builder.append(")");

        return builder.toString();
    }

    private void transform(StringBuilder builder, Object... parts) {
        for (var part : parts) {
            builder.append(" ");
            if (part instanceof Expr) {
                builder.append(((Expr) part).accept(this));
            } else if (part instanceof Stmt) {
                builder.append(((Stmt) part).accept(this));
            } else if (part instanceof Token) {
                builder.append(((Token) part).lexeme());
            } else if (part instanceof List) {
                transform(builder, ((List) part).toArray());
            } else {
                builder.append(part);
            }
        }
    }
}
