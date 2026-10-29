namespace analysis {
class Model {
public:
    int run() {
        return 1;
    }
};
struct Data {
    int size;
};
enum class Mode {
    Active,
};
int evaluate() {
    return 0;
}
}
template<typename T>
class Box {
};
template<>
class Box<int> {
};
template<typename T>
struct Holder {
};
template<>
struct Holder<int> {
};
namespace outer::inner {
enum class Nested {
    Value,
};
}
int analysis::Model::extra() {
    return 2;
}
namespace analysis { struct Forward; }
struct analysis::Forward {
    int value;
};
